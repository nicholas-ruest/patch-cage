use serde::{Deserialize,Serialize}; use std::{fs::{self,OpenOptions},io::{BufRead,BufReader,Write},path::PathBuf}; use uuid::Uuid;
#[derive(Clone)] pub struct App { store:PathBuf }
#[derive(Debug,Clone,Serialize,Deserialize)] pub struct Assessment { pub id:String,pub status:String,pub risk:f64,pub correlation_id:String,pub diff:String,pub receipts:Vec<pc_types::StageReceipt>,pub effects:Vec<String> }
#[derive(Debug,Deserialize)] pub struct Analyze { pub diff:String }
impl App { pub fn new(path:impl Into<PathBuf>)->Self{let p=path.into();if let Some(d)=p.parent(){let _=fs::create_dir_all(d);}Self{store:p}} fn append(&self,a:&Assessment)->anyhow::Result<()> {let mut f=OpenOptions::new().create(true).append(true).open(&self.store)?;writeln!(f,"{}",serde_json::to_string(a)?)?;Ok(())} pub fn get(&self,id:&str)->anyhow::Result<Option<Assessment>>{if !self.store.exists(){return Ok(None)} let f=OpenOptions::new().read(true).open(&self.store)?;let mut found=None;for l in BufReader::new(f).lines(){let a:Assessment=serde_json::from_str(&l?)?;if a.id==id{found=Some(a)}}Ok(found)} pub async fn analyze(&self,diff:&str,role:&str,cid:&str)->anyhow::Result<Assessment>{if role!="reviewer"&&role!="operator"{anyhow::bail!("policy_denied")} let receipts=vec![pc_ingest::evaluate(diff),
            pc_repository::evaluate(diff),
            pc_change_graph::evaluate(diff),
            pc_policy::evaluate(diff),
            pc_authz::evaluate(diff),
            pc_approval::evaluate(diff),
            pc_sandbox_plan::evaluate(diff),
            pc_sandbox_runner::evaluate(diff),
            pc_risk_engine::evaluate(diff),
            pc_ruvector::evaluate(diff),
            pc_rvf_ledger::evaluate(diff),
            pc_ruflo::evaluate(diff),
            pc_metaharness::evaluate(diff),
            pc_rune::evaluate(diff),
            pc_synthlang::evaluate(diff),
            pc_openfeature::evaluate(diff),
            pc_telemetry::evaluate(diff),
            pc_review::evaluate(diff),
            pc_release::evaluate(diff),
            pc_audit::evaluate(diff)]; let risk=receipts.iter().map(|r|r.score).sum::<f64>()/receipts.len() as f64; let a=Assessment{id:Uuid::new_v4().to_string(),status:"awaiting_review".into(),risk,correlation_id:cid.into(),diff:diff.into(),receipts,effects:vec![]};self.append(&a)?;Ok(a)} pub fn decide(&self,id:&str,approve:bool,role:&str)->anyhow::Result<Assessment>{if role!="reviewer"{anyhow::bail!("policy_denied")}let mut a=self.get(id)?.ok_or_else(||anyhow::anyhow!("not_found"))?;if approve{a.status="approved".into();a.effects.push("release_staged".into())}else{a.status="denied".into()}self.append(&a)?;Ok(a)} }
#[cfg(test)] mod tests{use super::*;#[tokio::test]async fn denial_has_no_effect(){let p=std::env::temp_dir().join(format!("pc-{}.jsonl",Uuid::new_v4()));let app=App::new(&p);assert!(app.analyze("diff","viewer","c").await.is_err());assert!(!p.exists());} #[tokio::test]async fn survives_reopen(){let p=std::env::temp_dir().join(format!("pc-{}.jsonl",Uuid::new_v4()));let a=App::new(&p).analyze("diff --git a/a b/a","reviewer","c").await.unwrap();assert!(App::new(&p).get(&a.id).unwrap().is_some());}}
