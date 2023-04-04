{% set data = load_data(path=path, format="plain") -%}
```{{ source_type }}
{% if start_line  %}{% if end_line  %}{{ data | split(pat="\n") | slice(start=start_line, end=end_line) | join(sep="") | linebreaksbr }}{% else %}{{ data | split(pat="\n") | slice(start=start_line) | join(sep="") | linebreaksbr}}{% endif %}{% else %}{% if end_line  %}{{ data | split(pat="\n")| slice(start=end_line) | join(sep="") | linebreaksbr }}{% else %}{{ data | trim }}{% endif %}{% endif %}
```
