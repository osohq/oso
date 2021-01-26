package oso

// import (
// 	"io/ioutil"
// 	"os"
// 	"path/filepath"
// 	"testing"

// 	yaml "github.com/goccy/go-yaml"
// )

// func testFromFile(t *testing.T, path string) {
// 	yamlInput, err := ioutil.ReadFile(path)
// 	if err != nil {
// 		t.Error(err)
// 	}
// 	var testCase TestCase
// 	err = yaml.Unmarshal(yamlInput, &testCase)
// 	if err != nil {
// 		t.Fatal(err)
// 	}
// 	testCase.RunTest(t)
// }

// func TestAll(t *testing.T) {
// 	err := filepath.Walk("../../../test/spec/", func(path string, info os.FileInfo, err error) error {
// 		if err != nil {
// 			return err
// 		}
// 		if info.IsDir() {
// 			return nil
// 		}
// 		testFromFile(t, path)
// 		return nil
// 	})
// 	if err != nil {
// 		t.Fatal(err)
// 	}
// }
