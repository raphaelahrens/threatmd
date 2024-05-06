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

```python
    <pytm condition in python code>
```

## Prerequisites

<Description of the prerequisites>

## References

- <A list of >
- <References>
```

## Example

```markdown
---
sid: DO04
severity: Medium
target: ['Dataflow']
likelihood: High
---

# XML Entity Expansion

An attacker submits an XML document to a target application where the XML document uses nested entity expansion to produce an excessively large output XML. XML allows the definition of macro-like structures that can be used to simplify the creation of complex structures. However, this capability can be abused to create excessive demands on a processor's CPU and memory. A small number of nested expansions can result in an exponential growth in demands on memory.

## Example

The most common example of this type of attack is the many laughs attack (sometimes called the 'billion laughs' attack). For example:

```xml
<?xml version=1.0?><!DOCTYPE lolz [<!ENTITY lol lol><!ENTITY lol2 &lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;><!ENTITY lol3 &lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;><!ENTITY lol4 &lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;><!ENTITY lol5 &lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;><!ENTITY lol6 &lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;><!ENTITY lol7 &lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6><!ENTITY lol8 &lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;><!ENTITY lol9 &lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;> ]><lolz>&lol9;</lolz>
```

This is well formed and valid XML according to the DTD. Each entity increases the number entities by a factor of 10. The line of XML containing `lol9;` expands out exponentially to a message with 10^9 entities. A small message of a few KBs in size can easily be expanded into a few GB of memory in the parser. By including 3 more entities similar to the lol9 entity in the above code to the DTD, the program could expand out over a TB as there will now be 10^12 entities. Depending on the robustness of the target machine, this can lead to resource depletion, application crash, or even the execution of arbitrary code through a buffer overflow.

## Mitigations

Design: Use libraries and templates that minimize unfiltered input. Use methods that limit entity expansion and throw exceptions on attempted entity expansion.Implementation: Disable altogether the use of inline DTD schemas in your XML parsing objects. If must use DTD, normalize, filter and white list and parse with methods and routines that will detect entity expansion from untrusted sources.

## Condition

```python
any(d.format == 'XML' for d in target.data) and target.handlesResources is False
```

## Prerequisites

This type of attack requires that the target must receive XML input but either fail to provide an upper limit for entity expansion or provide a limit that is so large that it does not preclude significant resource consumption.

## References

- https://capec.mitre.org/data/definitions/197.html
- http://cwe.mitre.org/data/definitions/400.html
- http://cwe.mitre.org/data/definitions/770.html
```
