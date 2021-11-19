## Runtimes

{%- for runtime in srtool %}
### {{ runtime.name | capitalize }}:
```
🏋️ Runtime Size:            {{runtime.data.runtimes.compressed.subwasm.size | filesizeformat }}
🎁 Metadata version:        V{{runtime.data.runtimes.compressed.subwasm.metadata_version }}
🔥 Core Version:            {{runtime.data.runtimes.compressed.subwasm.core_version }}
🗳️ system.setCode hash:     {{runtime.data.runtimes.compressed.subwasm.proposal_hash }}
```

{% endfor %}
