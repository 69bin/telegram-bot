use chrono::NaiveDateTime;
use chrono::Local;
use crate::get_mysql;

#[derive(Debug,Default,Clone)]
pub struct Group{
    username : String,
    user_id : String,
    group_id : String,
    join_time : NaiveDateTime,
    status : i8
}

impl Group {
    pub fn new(username:&str,user_id:&str,group_id:&str,status:i8) -> Group{
        let now = Local::now();
        let date_time = NaiveDateTime::new(now.date_naive(), now.time());
        Group { username: username.to_string(), user_id: user_id.to_string(), group_id: group_id.to_string(), join_time: date_time, status: status }
    }
}


//增加数据
pub async fn add_group_user(group : Group) -> Result<(),sqlx::Error>{
    let mysql = get_mysql();
    let str_time = group.join_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let sql = format!("
    insert into dd_group
    (username,user_id,group_id,join_time,status)
    values
    ('{}','{}','{}','{}',{})
    ",group.username,group.user_id,group.group_id,str_time,group.status);
    sqlx::query(&sql).execute(mysql).await?;
    Ok(())
}

