# Routes

### POST `/gitlab/:id`
> Requires valid token

### GET `/api/hooks`
> Requires auth

```json
[{
    "id": Integer,
    "description": String,
    "gitlab_token": String,
    "discord_url": String
}]
```

### PUT `/api/hook`
> Requires auth

```json
{
    "description": String,
    "gitlab_token": String,
    "discord_url": String
}
```

### DELETE `/api/hook/:id`
> Requires auth
