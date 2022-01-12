package oso_test

import (
 // "encoding/json"
//  "fmt"
//  "reflect"
  "testing"

  "gorm.io/gorm"
  "gorm.io/driver/sqlite"

  oso "github.com/osohq/go-oso"
  osoTypes "github.com/osohq/go-oso/types"
)

type GormAdapter struct {
  db *gorm.DB
}

func (a GormAdapter) BuildQuery(f *osoTypes.Filter) (interface{}, error) {
  return nil, nil
}

func (a GormAdapter) ExecuteQuery(q interface{}) (interface{}, error) {
  return nil, nil
}

func gormAdapter(db *gorm.DB) GormAdapter {
  return GormAdapter{db}
}

type Planet struct {
  gorm.Model
  Name string
}

type Sign struct {
  gorm.Model
  Name string
  Element string
  PlanetID uint
}

type Person struct {
  gorm.Model
  Name string
  SignID uint
}

func gormDb(dbFile string) *gorm.DB {
  db, _ := gorm.Open(sqlite.Open(dbFile), &gorm.Config{})
  db.AutoMigrate(&Planet{})
  db.AutoMigrate(&Sign{})
  db.AutoMigrate(&Person{})

  db.Create(&Planet{Name: "mars"})    // 1
  db.Create(&Planet{Name: "venus"})   // 2
  db.Create(&Planet{Name: "mercury"}) // 3
  db.Create(&Planet{Name: "moon"})    // 4
  db.Create(&Planet{Name: "sun"})     // 5
  db.Create(&Planet{Name: "jupiter"}) // 6
  db.Create(&Planet{Name: "saturn"})  // 7
  db.Create(&Planet{Name: "pluto"})   // 8

  db.Create(&Sign{Name: "aries", Element: "fire", PlanetID: 1})       // 1
  db.Create(&Sign{Name: "taurus", Element: "earth", PlanetID: 2})     // 2
  db.Create(&Sign{Name: "gemini", Element: "air", PlanetID: 3})       // 3
  db.Create(&Sign{Name: "cancer", Element: "water", PlanetID: 4})     // 4
  db.Create(&Sign{Name: "leo", Element: "fire", PlanetID: 5})         // 5
  db.Create(&Sign{Name: "virgo", Element: "earth", PlanetID: 3})      // 6
  db.Create(&Sign{Name: "libra", Element: "air", PlanetID: 2})        // 7
  db.Create(&Sign{Name: "scorpio", Element: "water", PlanetID: 1})    // 8
  db.Create(&Sign{Name: "sagittarius", Element: "fire", PlanetID: 6}) // 9
  db.Create(&Sign{Name: "capricorn", Element: "earth", PlanetID: 7})  // 10
  db.Create(&Sign{Name: "aquarius", Element: "air", PlanetID: 7})     // 11
  db.Create(&Sign{Name: "pisces", Element: "water", PlanetID: 6})     // 12

  db.Create(&Person{Name: "robin", SignID: 8})
  db.Create(&Person{Name: "pat", SignID: 2})
  db.Create(&Person{Name: "mercury", SignID: 6})
  db.Create(&Person{Name: "terry", SignID: 7})
  db.Create(&Person{Name: "chris", SignID: 11})
  db.Create(&Person{Name: "leo", SignID: 5})
  db.Create(&Person{Name: "eden", SignID: 4})
  db.Create(&Person{Name: "dakota", SignID: 10})
  db.Create(&Person{Name: "charlie", SignID: 1})
  db.Create(&Person{Name: "alex", SignID: 3})
  db.Create(&Person{Name: "sam", SignID: 12})
  db.Create(&Person{Name: "avery", SignID: 9})

  return db
}

func TestDataFiltering(t *testing.T) {

  o, _ := oso.NewOso()
  o.SetDataFilteringAdapter(GormAdapter{gormDb("data_filtering_test.sqlite")})
}
