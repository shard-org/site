pub fn get(UrlCollect(url): UrlCollect, Query(files): Query<Files>) -> Option<Html> {
    let mut url = url.join("/");

    if url.is_empty() {
        url = String::from("index");
    }
    
    let doc = files.get(&format!("files/doc/{url}.html"));
    let doc = doc.as_ref()
        .map(|f| String::from_utf8_lossy(f))
        .map_err(|e| {println!("{e}: {url}"); e})
        .ok()?;
    
    Some(Html(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
   <meta charset="UTF-8">
   <link rel="stylesheet" type="text/css" href="/styles/styles.css">
   <link rel=icon href="data:,">
   <title>Shard smarts!</title>

   <style>
      .sidebar {{
         width: 18ch;
         border-right: 1px solid var(--hint);
         padding-right: 8px;
         margin-right: 10px;
      }}
      .sidebar > h2 {{
         padding-bottom: 2px;
         padding-top: 0;
         margin-top: 0;
      }}

      .sidebar-category {{
         display: flex;
         flex-direction: column;
         margin-bottom: 15px;
      }}
      .sidebar-category > a {{
         padding: 3px 0;
         border: none;
         transition: background-color 0.2s ease;
      }}
      .sidebar-category > a:hover {{
         background-color: var(--base-acc);
      }}

      ul {{
         padding: 0;
         margin: 0;
         margin-bottom: 2px;
         list-style-type: none;
      }}
   </style>
</head>

<body>
   <header class=center style="margin-top: 15px;">
      <nav>
         <a href="/">Home</a>
         <a href="/doc">Documentation</a>
         <a href="/projects">Projects</a>
         <a href="/blog">Blog</a>
         <a href="/community">Community</a>
      </nav>
   </header>

   <div style="display: flex">
      <nav class=sidebar>
         <h2>Guide</h2>
         <div class=sidebar-category>
            <a href="/doc/guide/getting-started">Getting Started</a>
         </div>

         <h2>Examples</h2>
         <div class=sidebar-category>
            <a href="/doc/example/hello-world">Hello World</a>
            <a href="/doc/example/fibonacci">Fibonacci</a>
            <a href="/doc/example/bubble-sort">Bubble Sort</a>
            <a href="/doc/example/fat-pointers">Fat Pointers</a>
            <a href="/doc/example/iterators">Iterators</a>
            <a href="/doc/example/python">Python</a>
         </div>

         <h2>Specification</h2>
         <div class=sidebar-category>
            <a href="/doc/spec/tags">Tags</a>
            <a href="/doc/spec/types">Types</a>
            <a href="/doc/spec/labels">Labels</a>
            <a href="/doc/spec/literals">Literals</a>
            <a href="/doc/spec/memory">Memory</a>
            <a href="/doc/spec/operations">Operations</a>
            <a href="/doc/spec/other">Other</a>
         </div>
      </nav>

      <div style="margin: 0; padding-left:4px">
         {doc}
      </div>
   </div>

   <div class="center footer">
      <p>CC0 Public Domain 2024 Shard Team<p>
   </div>
</body>
</html>"#)))
}
