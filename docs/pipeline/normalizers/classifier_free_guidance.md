#

## `classifier_free_guidance`

The normalizer implements classifier-free guidance(CFG) with support for one main state, one CFG state with dynamic gamma(optional) and multiple CFG states with static gammas.

The main state is always the first state in the `infer` command, the dynamic gamma state, if any, is always the second state in the `infer` command, and the remaining states corresponds to static_gammas in order.

final logits = (1-gammas[0]-...-gammas[n-1])*states[0]+gammas[0]*states[1]+...+gammas[n-1]*states[n]

**Ensure the number of states match the number of gammas, or the axum server will PANIC!**

For example,  

```jsonc
{
    "static_gammas": [-2]
}
```

means there are two states, `states[0]` is the main state and `states[1]` is a CFG state with `gamma=-2`.

For example,  

```jsonc
{
    "static_gammas": [-2],
    "dynamic_gamma":{
        "min":0.2,
        "max":0.4,
        "threshold":0.01
        }
}
```

means there are three states, `states[0]` is the main state, `states[1]` is a CFG state with dynamic gamma(see below) and `states[2]` is a CFG state with `gamma=-2`.

#### Params

```jsonc
{
    // the number of states with static gammas.
    "static_gammas": [0.3,0.1,5,-2],
    // the parameters for dynamic gamma
    "dynamic_gamma":{
        // the minimum gamma value
        "min":0.2,
        // the maximum gamma value
        "max":0.4,
        // We know max_entropy is the entropy of a uniform distribution over the vocabulary.
        // if and only if the entropy of main state's logits 
        // minus the entropy of the CFG state's logits is greater than the threshold,
        // gamma = min + (max - min) * (1.0 - CFG_state_entropy / max_entropy)
        // otherwise gamma=0 
        "threshold":0.01
        }
}
```
