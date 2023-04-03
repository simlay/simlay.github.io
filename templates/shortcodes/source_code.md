{% set data = load_data(path=path, format="plain") -%}
```{{ source_type }}
{% if start_line  %}
{% if end_line  %}
{{ data | trim | slice(start=start_line, end=end_line) }}
{% else %}
{{ data | trim | slice(start=start_line) }}
{% endif %}
{% else %}
{% if end_line  %}
{{ data | trim | slice(start=end_line) }}
{% else %}
{{ data | trim }}
{% endif %}
{% endif %}
```
