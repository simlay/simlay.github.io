{% set data = load_data(path=path, format="plain") -%}
```{{ source_type }}
{% if start_line  %}{% if end_line  %}{{ data | split(pat="\n") | slice(start=start_line, end=end_line) | join(sep="
```