package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"os"

	"github.com/algolia/algoliasearch-client-go/v3/algolia/opt"
	"github.com/algolia/algoliasearch-client-go/v3/algolia/search"
	"github.com/bitly/go-simplejson"
	"github.com/hashicorp/go-hclog"
)

var (
	logger = hclog.New(&hclog.LoggerOptions{
		Level: hclog.Info,
		Color: hclog.AutoColor,
	})
	verbose              = flag.Bool("v", false, "Whether to output debugging information")
	algoliaApplicationID = flag.String("app_id", os.Getenv("ALGOLIA_APPLICATION_ID"), "Algolia Application ID")
	algoliaAdminAPIKey   = flag.String("key", os.Getenv("ALGOLIA_ADMIN_API_KEY"), "Algolia Admin API KEY")
)

func main() {
	// load up configuration variables
	flag.Parse()
	// args := flag.Args()

	if algoliaApplicationID == nil || *algoliaApplicationID == "" {
		logger.Error("Missing Algolia Application ID", "algoliaApplicationID", algoliaApplicationID)
		flag.Usage()
		return
	}
	if algoliaAdminAPIKey == nil || *algoliaAdminAPIKey == "" {
		logger.Error("Missing Algolia Admin API Key", "algoliaAdminAPIKey", algoliaAdminAPIKey)
		flag.Usage()
		return
	}
	if *verbose {
		logger.SetLevel(hclog.Debug)
		logger.Info("Setting logging level to DEBUG")
	}

	client := search.NewClient(*algoliaApplicationID, *algoliaAdminAPIKey)
	index := client.InitIndex("OSODOCS")

	params := []interface{}{
		opt.AttributesToRetrieve("objectID", "lang"),
		opt.HitsPerPage(5),
	}

	res, err := index.Search("pip", params...)
	if err != nil {
		panic(err)
	}
	fmt.Printf("ok %v", res)

	// uniqueRecords, err := processFiles(args)
	// if err != nil {
	// 	logger.Error("There was an error processing files", "error", err)
	// 	return
	// }

	// if err := writeCacheFile(uniqueRecords); err != nil {
	// 	logger.Error("There was an error writing cache file", "error", err)
	// 	return
	// }
	// var records []interface{}
	// for url, value := range uniqueRecords {
	// 	logger.Info("Writing it", "url", url)
	// 	records = append(records, value)
	// }
	// if err := writeIndexFile(records); err != nil {
	// 	logger.Error("Error writing index file", "error", err)
	// 	return
	// }
}

func writeCacheFile(uniqueRecords map[string]interface{}) error {
	logger.Info("Writing cache file")
	dump, err := json.MarshalIndent(uniqueRecords, "", "    ")
	if err != nil {
		logger.Error("There was an error marshalling into JSON", "error", err)
		return err
	}
	if err := ioutil.WriteFile("cache.json", dump, 0644); err != nil {
		logger.Error("There was an error writing the JOSN file", "error", err)
		return err
	}
	return nil
}

func writeIndexFile(records []interface{}) error {
	logger.Info("Writing index file")
	dump, err := json.MarshalIndent(records, "", "    ")
	if err != nil {
		logger.Error("There was an error marshalling into JSON", "error", err)
		return err
	}
	if err := ioutil.WriteFile("index.json", dump, 0644); err != nil {
		logger.Error("There was an error writing the JOSN file", "error", err)
		return err
	}
	return nil
}

func processFiles(args []string) (urls map[string]interface{}, err error) {
	urls = make(map[string]interface{})

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
			switch vals["kind"] {
			case "taxonomy", "home", "project":
				// logger.Debug("Skipping record", "kind", vals["kind"], "permalink", vals["permalink"])
			case "page", "section":
				if urls[permalink] == nil {
					logger.Debug("Including record", "kind", vals["kind"], "permalink", vals["permalink"])
					urls[permalink] = vals
				}

			default:
				return nil, fmt.Errorf("Not sure what to do with %s\n", vals["kind"])
			}
		}
	}
	return urls, nil
}
