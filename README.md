# fyin
> Open source alternative to Perplexity AI with ability to run locally (except search).

## Motivation
Perplexity AI is a great tool but it is not possible to run it locally and lacks customisations. This project is an attempt to build a similar tool that can be run locally, open source and faster than perplexity in bringing answers.

## Features
- [x] Run locally using ollama or use openai API
- [x] local VectorDB for fast search
- [x] very quick searching, scrapping & answering due to parallelism 
- [x] Configurable number of search results to parse
- [x] local scrapping of websites

---

## Demo

[![Demo](https://github.com/shadowfax92/fyin.app/blob/a08e723d7622ab0115769443e8c055ba50ff06db/public/thumbnail.png)](https://youtu.be/gIjgus8jgko)

---

## Installation

1. Clone the repository - `git clone https://github.com/shadowfax92/fyin.app`
2. Get Bing API key
3. Get OpenAI API key or [Ollama](https://ollama.com/)
4. Fill/setup the environment variables (see `sample.env` file, copy it to `.fyin.env` and fill the values))
5. `cargo run --query "<Question>" -n <number of search results>`


### Environment Variables
```
# Open AI config
OPENAI_KEY = "<OPENAI_API_KEY>"
OPENAI_BASE_URL = "" # Leave blank for default
BING_SUBSCRIPTION_KEY = "<BING_SUBSCRIPTION_KEY>"
BING_ENDPOINT = "" # Leave blank for default
EMBEDDING_MODEL_NAME = "text-embedding-ada-002"
CHAT_MODEL_NAME = "gpt-4o"

# Ollama config
# OPENAI_KEY = "ollama"
# OPENAI_BASE_URL = "http://localhost:11434/v1"
# BING_SUBSCRIPTION_KEY = "<BING_SUBSCRIPTION_KEY>"
# BING_ENDPOINT = ""
# EMBEDDING_MODEL_NAME = "llama3"
# CHAT_MODEL_NAME = "llama3"
```

---

## TODO
- [ ] Simlar to perplexity.ai, use GPT to figure out 3-5 search queries based on prompt
  - This should give better results as we are translating human query into search query.
- [ ] Build a simple website
- [ ] Hosted version of the app
