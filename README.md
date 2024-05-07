threatrs is a tool to transform Markdown threats in the threatrs format to a threats.json file for pytm.

# Running threatrs

```shell
threats_pytm <directory of markdown threats>
```

Prints the threats in pytms JSON format to stdout.


# threatrs format

The Markdown files for each threat needs to have a metadata field containing following attributes

```markdown
---
sid: <SID>                    # A string containing the id identifying the threat 
severity: Medium              # The severity for this threat, either "Low", "Medium", "High" or "Very high"
target: [<Target Classes>]    # The targeted elements ("Process", "Datastore", "Dataflow", "ExternalEntity")
likelihood: Low               # The likelihood of this threat happening "Low", "Medium", "High" or "Very high"
---

# <Name of the Threat>

<Description of the threat>

## Example

<Examples how the threat can be introduced and exploited>

## Mitigations

<Description of how the threat can be mitigated>

## Condition

`.``python
    <pytm condition in python code>
`.``

## Prerequisites

<Description of the prerequisites>

## References

- <A list of >
- <References>
```

## Example

[Example threat](test_examples/DO04.md)
