#

## `sliding_penalty`

This logits transformer will record input tokens and penaltize the logits distribution. Unlike `global_penalty`, this transformer will only keep most recent `window_size` tokens in record.

#### Params

```jsonc
{
    // Occurrence penalty of the penalty.
    "alpha_occurrence": 0.3,
    // Presence penalty of the penalty.
    "alpha_presence": 0.3,
    // the maximum number of tokens in record
    "window_size":256,
    // Penalty mode.
    // The default mode is "Subtract", which means for each token,
    // result_logit=initial_logit-alpha_occurrence*(the number of the token recorded)-(the number of the token recorded is greater than 0 ? alpha_presence:0)
    // Another possible mode is "Divide", which means for each token,
    // if initial_logit>=0 then
    //      result_logit=initial_logit/(alpha_occurrence*(the number of the token recorded))/(the number of the token recorded is greater than 0 ? alpha_presence:1)
    // else
    //      result_logit=initial_logit*alpha_occurrence*(the number of the token recorded)*(the number of the token recorded is greater than 0 ? alpha_presence:1)
    "mode": "Subtract"
}
```
