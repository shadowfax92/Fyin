# fyin
> Open source alternative to Perplexity AI with ability to run locally (except search).

## Features
- [x] Run locally using ollama or use openai API
- [x] local VectorDB for fast search
- [x] very quick searching, scrapping & answering due to parallelism 
- [x] Configurable number of search results to parse
- [x] local scrapping of websites

## Demo

[![Demo](https://github.com/shadowfax92/fyin.app/blob/a08e723d7622ab0115769443e8c055ba50ff06db/public/thumbnail.png)](https://youtu.be/gIjgus8jgko)

## Installation

1. Clone the repository - `git clone https://github.com/shadowfax92/fyin.app`
2. Get Bing API key
3. Get OpenAI API key or [Ollama](https://ollama.com/)
4. Fill/setup the environment variables (see `sample.env` file)
5. `cargo run --query "<Question>" -n <number of search results>`


## TODO & Future Plans
- [ ] Simlar to perplexity.ai, use GPT to figure out 3-5 search queries based on prompt
  - This should give better results as we are translating human query into search query.
- [ ] Build a simple website
- [ ] Hosted version of the app
