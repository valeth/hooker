# Hooker

A webhook server to forward events to Discord.


## Usage

You need to supply a user if you want to access the API and modify hooks at runtime.

```sh
# Has to be a SHA256 hash of the plain text password
cargo run -- --user USERNAME:PWHASH
```


## API

The server comes with a simple API to manage webhooks.

The API uses [Basic HTTP Authentication]

### `GET /api/hooks`
> (!) Requires authentication

Response payload:
```json
[{
    "id": "String",
    "description": "String",
    "gitlab_token": "String",
    "discord_url": "URI",
    "created_at": "DateTime"
}]
```

### `POST /api/hook`
> (!) Requires authentication

Request payload:
```json
{
    "description": "String",
    "gitlab_token": "String",
    "discord_url": "URI"
}
```

Response payload:
```json
{
    "id": "String",
    "description": "String",
    "gitlab_token": "String",
    "discord_url": "URI"
}
```

### `DELETE /api/hook/:id`
> (!) Requires authentication

Request Parameters:
```
id: String
```

### `POST /hooks/gitlab/:id`
> (!) Requires valid token

Request Parameters:
```
id: String
```


> **Note**: The GitLab webhook test events provide a different payload than actual events.
> Because of this some event hooks might not seem to work, because they are missing JSON fields.


<!-- links -->

[Basic HTTP Authentication]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication#Basic_authentication_scheme
