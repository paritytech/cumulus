
{%- if c.meta.C and c.meta.C.value >= 7-%}
{%- set prio = "❗️HIGH" -%}
{%- set txt = "You must upgrade as as soon as possible" -%}
{%- elif c.meta.C and c.meta.C.value >= 5 -%}
{%- set prio = "📣 Medium" -%}
{%- set txt = "You should upgrade in a timely manner" -%}
{%- elif c.meta.C and c.meta.C.value >= 3 -%}
{%- set prio = "📌 Low" -%}
{%- set txt = "You may upgrade at your convenience" -%}
{%- endif -%}

{%- if prio %}
{{ prio }} {{ txt }}
{%- else %}
<!-- No relevant Priority label as been detected -->
{%- endif %}
