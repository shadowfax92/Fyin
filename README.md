# fyin
> Open source alternative to Perplexity AI with ability to run locally. 

## Motivation
This project aims to build a tool that can be run locally, is open-source, and delivers faster answers, serving as an alternative to Perplexity AI.

## Features
- [x] Run locally using ollama or use openai API
- [x] local VectorDB for fast search
- [x] very quick searching, scrapping & answering due to parallelism 
- [x] Configurable number of search results to parse
- [x] local scrapping of websites

---

## Demo

[![Youtube Demo - Running locally with OpenAI gpt-4o](https://github.com/shadowfax92/fyin-website/blob/78f9785d4905151ac1faafc6ab4bc15076bbdcf0/public/demo.gif)](https://www.youtube.com/watch?v=9tVGcPokgdo)

(You can watch the demo on YouTube too [here](https://www.youtube.com/watch?v=9tVGcPokgdo))

---

## Installation

1. Clone the repository - `git clone https://github.com/shadowfax92/fyin.app`
2. Get Bing API key
3. Get OpenAI API key or [Ollama](https://ollama.com/)
4. Fill/setup the environment variables (see `sample.env` file, copy it to `.fyin.env` and fill the values))
5. `cargo run --query "<Question>" -n <number of search results>`


### Environment Variables
```
# Open AI config; Ollama config in comments

# OPENAI_API_KEY="ollama"
OPENAI_API_KEY="your-openai-api-key"

# OPENAI_BASE_URL=http://localhost:11434/v1
# Leave blank for default
OPENAI_BASE_URL=

BING_SUBSCRIPTION_KEY="your-bing-subscription-key"
# Leave blank for default
BING_ENDPOINT=

# EMBEDDING_MODEL_NAME="llama3"
EMBEDDING_MODEL_NAME="text-embedding-ada-002"

# CHAT_MODEL_NAME="llama3"
CHAT_MODEL_NAME="gpt-4o"
```

### Docker
Here is how you can run the app using docker:
1. Build the docker image - `docker build -t fyin .`
2. Create environment file - `cp sample.env .env` and populate the values
3. Run the docker container 

`docker run --rm --env-file .env fyin --query "<your question>" --search <optional: number of search results to parse>`

## Notes
- The app use Bing API for searching. You can get from [Active Bing API](https://www.microsoft.com/en-us/bing/apis/bing-web-search-api).
- You can get OpenAI API key form [OpenAI](https://openai.com/api/).
- [Ollama](https://www.ollama.com/) setup instructions here.

---

## TODO
- [ ] Simlar to perplexity.ai, use GPT to figure out 3-5 search queries based on prompt
  - This should give better results as we are translating human query into search query.
- [ ] Build a simple website
- [ ] Hosted version of the app
