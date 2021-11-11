{%- macro change(c) -%}

{%- if c.meta.C and c.meta.C.value >= 7-%}
{%- set prio = "❗️HIGH" -%}
{%- elif c.meta.C and c.meta.C.value >= 5 -%}
{%- set prio = "📣 Medium" -%}
{%- elif c.meta.C and c.meta.C.value >= 3 -%}
{%- set prio = "📌 Low" -%}
{%- else -%}
{%- set prio = "" -%}
{%- endif -%}

{%- if c.meta.D and c.meta.D.value == 1-%}
{%- set audit = "✅ audtited" -%}
{%- elif c.meta.D and c.meta.D.value == 2 -%}
{%- set audit = "✅ trivial" -%}
{%- elif c.meta.D and c.meta.D.value == 3 -%}
{%- set audit = "✅ trivial" -%}
{%- elif c.meta.D and c.meta.D.value == 5 -%}
{%- set audit = "⏳ pending non-critical audit" -%}
{%- else -%}
{%- set audit = "" -%}
{%- endif -%}


{{ audit }} [`#{{c.number}}`]({{c.html_url}}) {{ prio }} - {{ c.title | capitalize }}
{%- endmacro change %}
