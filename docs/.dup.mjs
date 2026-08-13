import http from 'node:http'; import fs from 'node:fs'; import path from 'node:path';
import { chromium } from 'playwright';
const DIST='/home/opencode/Codes/peregrine/docs/dist';
const MIME={'.html':'text/html','.css':'text/css','.js':'text/javascript','.svg':'image/svg+xml','.png':'image/png','.woff2':'font/woff2','.xml':'text/xml'};
const server=http.createServer((req,res)=>{const p=decodeURIComponent(new URL(req.url,'http://x').pathname);for(const c of [p,p+'.html',path.join(p,'index.html')]){const f=path.join(DIST,c);if(fs.existsSync(f)&&fs.statSync(f).isFile()){res.writeHead(200,{'content-type':MIME[path.extname(f)]||'application/octet-stream'});return fs.createReadStream(f).pipe(res);}}res.writeHead(404);res.end('nf');});
await new Promise(r=>server.listen(4399,r));
const browser=await chromium.launch();
const ctx=await browser.newContext({viewport:{width:1440,height:900},colorScheme:'dark'});
const page=await ctx.newPage(); await page.goto('http://localhost:4399/',{waitUntil:'networkidle'}); await page.waitForTimeout(400);
const r=await page.evaluate(()=>{
  const q=(s)=>[...document.querySelectorAll(s)];
  return {
    dotFields: q('#dot-field').length,
    noises: q('.noise').length,
    reveals: q('[data-reveal]').length,
    // 每个 data-reveal 元素是否是 <section>
    revealTags: q('[data-reveal]').map(e=>e.tagName+':'+(e.className||'').slice(0,30)),
    // 查 SectionHeading 内部结构（可能多一条线）
    shHTML: document.querySelector('.sh-heading')?.outerHTML?.slice(0,600),
    shChildren: [...(document.querySelector('.sh-heading')?.children||[])].map(c=>c.className),
    // 标题下横线元素清点
    h2Like: q('.lp-section h2').length,
  };
});
console.log(JSON.stringify(r,null,1));
await ctx.close(); await browser.close(); server.close();
