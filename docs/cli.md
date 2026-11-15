# CLI contract

CLI executable: `skrynia`. Two independent groups: `app` operates on local static hosting and GitHub sources, while `object`, `log`, `admin` operate on Skrynia durable data. App deployment must not require a log or a grant.

## Primary workflow

```sh
skrynia app deploy --name NAME --repo https://github.com/OWNER/REPO --subdir PATH --apps-dir DIR [--commit FULL_SHA] [--builder-image IMAGE]
skrynia app inspect --name NAME --apps-dir DIR
skrynia app releases --name NAME --apps-dir DIR
skrynia app rollback --name NAME --release ID --apps-dir DIR
```

Build runs `make build` inside the selected repo subdirectory in a disposable Podman builder; output is `build/`. Full checkout is accessible for shared monorepo modules. Static server maps `DIR/apps/NAME/` to `/apps/NAME/`. Every release has immutable metadata including exact resolved commit. Rollback requires a retained release and never affects Skrynia stored data.

## Optional data commands

```sh
skrynia object get KEY --url URL [--output FILE]
skrynia object create KEY --url URL --token-file PRIVATE_FILE --file FILE
skrynia object replace KEY --url URL --token-file PRIVATE_FILE --file FILE [--if-sha256 HASH]
skrynia object delete KEY --url URL --token-file PRIVATE_FILE [--if-sha256 HASH]
skrynia object list MATCHER --url URL --token-file PRIVATE_FILE
skrynia admin app plan --name NAME --max-bytes N --max-objects N
skrynia admin app create --name NAME --url URL --parent-token-file PRIVATE_FILE --credentials-out PRIVATE_FILE --max-bytes N --max-objects N
skrynia log init PREFIX --url URL --token-file PRIVATE_FILE
skrynia log append PREFIX --url URL --token-file PRIVATE_FILE --file FILE
skrynia log read PREFIX --url URL
```

API operations preserve Skrynia's create, modify and delete distinction. Credentials are not printed in normal terminal output. Admin provisioner's owner grant is optional and contains **no** assumed protocol keys. Specific protocol management, task queues and Wasm transport are future additions, not automatic effects of deployment.

## Failure contracts

Do not print secrets or silently repeat remote writes with uncertain outcomes. Symlink activation occurs only after verifying the staged build. A failed build must leave the previous active app untouched. Do not treat a paginated listing as an atomic snapshot.
