#

## `until`

A terminal which will terminate the generation when the result *contains* the given content.

Note that the terminal does not affect sampling, so there's no guarantee that the content will at the *end* of the result.

#### Params

```jsonc
{
    "until": ...,
    // A cap for generation so it won't stuck for too long (Optional)
    "cap": 128
}
```
