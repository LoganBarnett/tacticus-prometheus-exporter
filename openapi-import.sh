#!/usr/bin/env bash

set -euo pipefail

# Run it through `jq` to pretty print it, in case we need to take a look at it.
curl --request GET 'https://api.tacticusgame.com/api-docs' \
  | jq '.' \
  > openapi.json
jq '
  # For each path and method, strip "X-API-KEY" header parameters
  .paths |= with_entries(
    .value |= with_entries(
      .value |= if has("parameters") then
        .parameters |= map(
          select(.name != "X-API-KEY" or .in != "header")
        )
      else . end
    )
  ) |

  # Add security scheme under components
  .components.securitySchemes.ApiKeyAuth = {
    type: "apiKey",
    name: "X-API-KEY",
    in: "header"
  } |

  # Add global security requirement
  .security = [{"ApiKeyAuth": []}]
' openapi.json > openapi-security-fix.json
jq '
  .components.schemas."Xp Book".properties.id.enum[0] = "xpCommon"
' openapi-security-fix.json > openapi-final.json

# Validation is skipped because there are known errors with the spec.  Primarily
# this is schema names containing spaces.
# The --global-property argument is given to prevent the generator from laying
# down a slew of Markdown documents.  I think this might be helpful if we were
# generating an API library, but that's beyond the scope of my endeavor.  Still,
# it would make such an endeavor pretty easy, and something that should be
# considered.
openapi-generator-cli generate \
  --input-spec openapi-final.json \
  --generator-name rust \
  --skip-validate-spec \
  --global-property models,apis
# But docs gets emitted anyways.  We'll have to break this into its own library
# to prevent this obnoxiousness from being disruptive.
rm -rf docs
