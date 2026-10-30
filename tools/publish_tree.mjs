#!/usr/bin/env node
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { spawnSync } from 'node:child_process';
const [root, repo, branch, message] = process.argv.slice(2);
if(!root||!repo||!branch||!message) throw new Error('usage: publish_tree ROOT REPO BRANCH MESSAGE');
const gh=(args,input)=>{const r=spawnSync('gh',args,{encoding:'utf8',input,maxBuffer:64*1024*1024});if(r.status!==0)throw new Error(r.stderr||r.stdout);return r.stdout.trim()};
const walk=d=>readdirSync(d).flatMap(n=>{if(['.git','target','node_modules','dist'].includes(n))return[];const p=join(d,n),s=statSync(p);return s.isDirectory()?walk(p):[p]});
const ref=JSON.parse(gh(['api',`repos/${repo}/git/ref/heads/${branch}`]));
const base=ref.object.sha;
const baseCommit=JSON.parse(gh(['api',`repos/${repo}/git/commits/${base}`]));
const entries=[];
for(const file of walk(root)){
  const content=readFileSync(file).toString('base64');
  const blob=JSON.parse(gh(['api',`repos/${repo}/git/blobs`,'--method','POST','--input','-'],JSON.stringify({content,encoding:'base64'})));
  const path=relative(root,file).replaceAll('\\','/');
  const executable=(path.startsWith('scripts/')&&(/\.(sh|mjs)$/.test(path)))||path==='tools/publish_tree.mjs';
  entries.push({path,mode:executable?'100755':'100644',type:'blob',sha:blob.sha});
}
const tree=JSON.parse(gh(['api',`repos/${repo}/git/trees`,'--method','POST','--input','-'],JSON.stringify({base_tree:baseCommit.tree.sha,tree:entries})));
const commit=JSON.parse(gh(['api',`repos/${repo}/git/commits`,'--method','POST','--input','-'],JSON.stringify({message,tree:tree.sha,parents:[base]})));
gh(['api',`repos/${repo}/git/refs/heads/${branch}`,'--method','PATCH','--input','-'],JSON.stringify({sha:commit.sha,force:false}));
console.log(commit.sha);
