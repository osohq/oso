package main

import (
	"flag"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"

	"github.com/algolia/algoliasearch-client-go/v3/algolia/opt"
	"github.com/algolia/algoliasearch-client-go/v3/algolia/search"
	"github.com/bitly/go-simplejson"
	"github.com/hashicorp/go-hclog"
)

var (
	algoliaApplicationID = flag.String("app_id", os.Getenv("ALGOLIA_APPLICATION_ID"), "Algolia Application ID")
	algoliaAdminAPIKey   = flag.String("key", os.Getenv("ALGOLIA_ADMIN_API_KEY"), "Algolia Admin API KEY")
	algoliaIndex         = flag.String("index", os.Getenv("ALGOLIA_INDEX"), "Algolia Index")
	logger               = hclog.New(&hclog.LoggerOptions{
		Level: hclog.Info,
		Color: hclog.AutoColor,
	})
	matchChangelog = regexp.MustCompile("project/changelogs")
	searchClient   *search.Client
	verbose        = flag.Bool("v", false, "Whether to output debugging information")
)

type Record struct {
	Content      string `json:"content"`
	Description  string `json:"description"`
	Kind         string `json:"kind"`
	Language     string `json:"language"`
	Permalink    string `json:"permalink"`
	RecordWeight int64  `json:"record_weight"`
	Section      string `json:"section"`
	Summary      string `json:"summary"`
	Title        string `json:"title"`
}

/*
	This processes the files passed in.

	1. Load up the configuration from the ENV and/or command line
	2. Read through the files in the public folder
	3. Process them and get the unique records
	4. Upload the records
*/
func main() {
	// load up configuration variables
	flag.Parse()

	if !loadConfiguration() {
		return
	}

	args, err := loadSearchFiles()
	if err != nil {
		logger.Error("There was an error loading search files", "error", err)
		return
	}

	uniqueRecords, err := processFiles(args)
	if err != nil {
		logger.Error("There was an error processing files", "error", err)
		return
	}

	var records []interface{}
	for _, value := range uniqueRecords {
		records = append(records, value)
	}

	logger.Info("Sending data to Algolia")
	// create algolia client
	searchClient = search.NewClient(*algoliaApplicationID, *algoliaAdminAPIKey)
	index := searchClient.InitIndex(*algoliaIndex)

	// send to algolia
	result, err := index.ReplaceAllObjects(records, opt.AutoGenerateObjectIDIfNotExist(true))
	if err != nil {
		logger.Error("There was an error sending the records to Algolia", "error", err)
	}
	if err := result.Wait(); err != nil {
		logger.Error("There was an error ingesting data", "error", err)
	}
}

func loadSearchFiles() ([]string, error) {
	fileList := make([]string, 0)

	if err := filepath.Walk("./public", func(path string, f os.FileInfo, err error) error {
		if f.IsDir() {
			return nil
		}
		if f.Name() == "search.json" {
			fileList = append(fileList, path)
		}
		return nil
	}); err != nil {
		logger.Error("There was an error walking the public directory", "error", err)
		return nil, err
	}
	return fileList, nil
}

func loadConfiguration() bool {
	if algoliaApplicationID == nil || *algoliaApplicationID == "" {
		logger.Error("Missing Algolia Application ID", "algoliaApplicationID", algoliaApplicationID)
		flag.Usage()
		return false
	}
	if algoliaAdminAPIKey == nil || *algoliaAdminAPIKey == "" {
		logger.Error("Missing Algolia Admin API Key", "algoliaAdminAPIKey", algoliaAdminAPIKey)
		flag.Usage()
		return false
	}
	if algoliaIndex == nil || *algoliaIndex == "" {
		logger.Error("Missing Algolia Index", "algoliaIndex", algoliaIndex)
		flag.Usage()
		return false
	}
	if *verbose {
		logger.SetLevel(hclog.Debug)
		logger.Info("Setting logging level to DEBUG")
	}
	return true
}

func processFiles(args []string) (urls []Record, err error) {
	uniqueRecords := make(map[string]*Record)

	for _, item := range args {
		contents, err := ioutil.ReadFile(item)

		if err != nil {
			logger.Error("Error loading file", "error", err)
			return nil, err
		}
		logger.Info("Processing JSON file", "name", item, "length", len(contents))

		doc, err := simplejson.NewJson(contents)
		if err != nil {
			logger.Error("Error processing JSON file", "error", err)
			return nil, err
		}

		for _, val := range doc.MustArray() {
			vals := val.(map[string]interface{})

			permalink := vals["permalink"].(string)

			if uniqueRecords[permalink] == nil {
				record := &Record{
					Content:      vals["content"].(string),
					Description:  vals["description"].(string),
					Kind:         vals["kind"].(string),
					Language:     vals["lang"].(string),
					Permalink:    permalink,
					RecordWeight: ProcessRecordWeight(vals),
					Section:      vals["section"].(string),
					Summary:      vals["summary"].(string),
					Title:        vals["title"].(string),
				}
				if record.RecordWeight > 0 {
					uniqueRecords[permalink] = record
				} else {
					logger.Debug("Skipping", "record", record)
				}
			}

		}
	}
	for _, record := range uniqueRecords {
		urls = append(urls, *record)
	}
	return urls, nil
}

func ProcessRecordWeight(vals map[string]interface{}) int64 {
	// we want to weight changelogs, so we will set their "kind"
	permalink := vals["permalink"].(string)
	if matchChangelog.MatchString(permalink) {
		logger.Debug("Found a changelog")
		// these will be included but at a low weight
		return 1
	}

	if vals["content"].(string) == "" {
		// Content is empty, don't index this page (happens for pages in language X
		// when hugo is building the search index for language Y)
		return 0
	}

	kind := vals["kind"].(string)
	section := vals["section"].(string)

	logger.Debug("ProcessKind", "kind", kind, "section", section, "permalink", permalink)

	switch kind {
	case "home":
		// We don't want to index the homepage
		return 0
	case "taxonomy":
		// this is a taxonomy page, like https://www.osohq.com/docs/oss/node/tags.html, so we ignore it
		return 0
	case "page":
		return 10
	case "section":
		switch section {
		case "getting-started":
			return 0
		case "guides", "learn", "reference", "project":
			return 10
		}

	default:
		logger.Error("I don't know how to handle this type of page - please update my code", "kind", kind, "section", section)
		os.Exit(1)
	}

	return 0
}
