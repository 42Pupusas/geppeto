# Geppeto

A Nostr based API for GPT-3, powered on Lightning. The bot listens for event `kind 29000` (inspired by [NIP-9000](https://bountsr.org/Public-Conversations-with-AI-Assistants/)) and will query the prompt to the GPT-3.5 API. The response is posted as a `kind 29001` event. Ephemeral events are used as **this** bot does not wish to keep a record of the prompts.

The bots are written in Rust and the [MVP client](https://geppeto.lat) is using the Yew framework.

## Tokens

OpenAI's API is **not a free product**, so this bot is powered by Nostr "tokens". A user requests an amount of tokens, the bot generates a lightning invoice (10 sat/token), and if paid, will publish the required amount of tokens to the relay. Every time a prompt is requested from the bot, the bot deletes a token from the relay. Users can query events with `kind 9777` and a `["p", <user_pubkey>]` to check how many tokens they have.

## Mission

This project provides a prototype for intergating AI agents to Nostr in a orderly and profitable manner. Specific event kinds can be standardized to identify AI messages across all clients. The bot can be adapted to contact any LLM Api, so hopefully more bots can start coming online using this template.

## Support the Project

I am an independent dev from El Salvador working a full time job and building open source applications. Support this project by donating to:

BC1QRZSUAC0N0KKWZS24RQ5P3NF0UH2ZPQ86ER55CS
