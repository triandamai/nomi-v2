# pen/ — what the gallery is held to

Snapshots taken from `Nomi.pen` (pen.dev), the source of truth for how frontend-new looks
(AGENTS.md §0). Nothing here is edited by hand.

| File | What | Used by |
|---|---|---|
| `bounds.json` | Every section of the three design sheets (offset from the sheet's DS Header, height) and every board inside them (x, y, width, height relative to its section), at the sheet's 1600 width. | `e2e/pen-parity.e2e.ts` (CI), `npm run pen:report` |
| `export/<sheet>--<section>.html` | The pen's own HTML export of each section. | `npm run pen:compare`, as the text-by-text reference |

The sheets: `nomi` = *Design System / Nomi* (`YOYH1`), `chat` = *Design System / Chat responses*
(`Ljvtr`), `build` = *Design System / Build* (`DcG0H`).

## Comparing

With the gallery running (`npm run dev`, port 5175; add `--url` for another):

```sh
npm run pen:report                                  # all sections and boards against bounds.json
npm run pen:compare -- build workspace              # every text in one section against export/
npm run pen:compare -- chat rich-content --all      # every line, not only where the offset changes
npm run pen:boxes -- nomi "section[aria-label=Rules] section" 2   # an element's boxes
```

`compare` renders the export in Chromium with local Geist and the pen's whole-pixel line heights,
and turns its side strokes into layout-free lines as the pen draws them. It still collapses
trailing spaces between inline runs and mis-sizes negative gaps and some fill widths: when a line
looks wrong, check the frame's own bounds in the pen (below) before changing a component.

## Refreshing, when the pen changes

Open `Nomi.pen` in the Pencil app and run these through the Pencil MCP `execute` tool (Claude Code:
`mcp__pencil__execute` with the document's path from `mcp__pencil__get_app_state`).

**bounds.json**: prints the JSON for `"sheets"`; replace that key in `bounds.json` and update
`"exported"`:

```js
const sheets={nomi:"YOYH1",chat:"Ljvtr",build:"DcG0H"}, out={};
for (const [key,id] of Object.entries(sheets)) { const sections=[], boards={}; let top=null;
  Get(id,(n,c)=>{ if(c.depth!==1) return; if(top===null) top=c.bounds.y; const name=n.name.replace(/^Section \/ /,""); sections.push([name, Math.round(c.bounds.y-top), Math.round(c.bounds.height)]); const list=[];
    Get(n.id,(b,bc)=>{ if(bc.depth>=1 && b.type==="frame" && /^Board/.test(b.name) && !/Head/.test(b.name)){ let x=0,y=0,cc=bc; while(cc && cc.depth>=1){ x+=cc.bounds.x; y+=cc.bounds.y; cc=cc.parentCtx; } let t=""; Get(b.id,(k)=>{ if(!t && k.type==="text") t=String(k.content); }); list.push([t.slice(0,40),Math.round(x),Math.round(y),Math.round(bc.bounds.width),Math.round(bc.bounds.height)]); bc.skipChildren(); } });
    if(list.length) boards[name]=list; c.skipChildren(); });
  out[key]={sections,boards}; }
Print(JSON.stringify(out));
```

**export/**: writes one HTML file per section (set `dir` to this folder's absolute path):

```js
const dir="/absolute/path/to/frontend-new/pen/export/", list=[];
for (const [key,s] of [["nomi","YOYH1"],["chat","Ljvtr"],["build","DcG0H"]]) {
  Get(s,(n,c)=>{ if(c.depth===1){ const slug=n.name.replace(/^Section \/ /,"").toLowerCase().replace(/[^a-z0-9]+/g,"-").replace(/^-|-$/g,""); list.push([n.id, key+"--"+slug]); c.skipChildren(); } }); }
for (const [id,name] of list) Export([id],"html-css",dir+name+".html",{includeLayerIds:true});
```

**One frame's exact layout** (what to read before building a component): its properties and
children, then their positions:

```js
Print(JSON.stringify(Get("<frame id>", {depth: 3, resolveInstances: true})));
Get("<frame id>", (n, c) => { if (c.depth <= 3) Print("  ".repeat(c.depth) + n.name, c.bounds); }, {resolveInstances: true});
```

Change the gallery in the same PR, so `npm run test:e2e` stays green.
