# Development

To get started:

1. `make install`
2. Install [Hugo][]
3. `make run`

# Search Index Generation

The search folder has a tiny Go program that can create an index from the Hugo files and send to Algolia for search

```bash
Usage of ./searcher:
  -app_id string
    	Algolia Application ID
  -index string
    	Algolia Index
  -key string
    	Algolia Admin API KEY
  -v	Whether to output debugging information
```

You can get API keys from Algolia at https://www.algolia.com/apps/{APPLICATION_ID}/api-keys/all
## Useful resources

- Tailwind Cheat Sheet: https://nerdcave.com/tailwind-cheat-sheet
- Tailwind docs: https://tailwindcss.com/docs/container
- Tailwind UI (sample components): https://tailwindui.com/preview

- Data relationships in Hugo: https://forestry.io/blog/data-relationships-in-hugo/

### Components

[Hugo]: https://gohugo.io/getting-started/installing
