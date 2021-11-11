{%- import "change.md" as m_c -%}

## Changes

You can find below the full list of the {{ cl.changes | length }} in this release.

<!-- <details><summary>Changes since statemine_v5</summary> -->
{% for pr in cl.changes | sort(attribute="merged_at") %}
- {{ m_c::change(c=pr) }}
{% endfor %}
<!-- </details> -->
