{%- import "change.md" as m_c -%}

# {{cl.repository.name | capitalize }}

Our changelog has {{ cl.changes | length }} commits.

{% for pr in cl.changes -%}
- {{ m_c::change(c=pr) }}
{% endfor %}

{%- include "runtimes.md" -%}

{%- include "docker_image.md" -%}
