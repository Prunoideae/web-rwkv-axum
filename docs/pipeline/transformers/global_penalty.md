#

## `global_penalty`

This logits transformer will record input tokens and penaltize the logits distribution. The penalty will not reset until an explicit call is done. So this is the `global` penalty.

#### Params

```jsonc
{
    // Occurrence penalty of the penalty.
    // Logits will be subtracted by this for each occurrence.
    "alpha_occurrence": 0.3,
    // Presence penalty of the penalty.
    // Logits will be subtracted by this if it occurred at
    // least once.
    "alpha_presence": 0.3
}
```
