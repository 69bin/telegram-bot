use chrono::NaiveDateTime;
use chrono::Local;
use sqlx::FromRow;
use crate::get_mysql;

#[derive(Debug,Default,Clone,FromRow)]
pub struct Group{
    username : String,
    user_id : String,
    group_id : String,
    join_time : NaiveDateTime,
    status : i8,
    join_count: Option<i32>
}

impl Group {
    pub fn new(username:&str,user_id:&str,group_id:&str,status:i8) -> Group{
        let now = Local::now();
        let date_time = NaiveDateTime::new(now.date_naive(), now.time());
        Group { username: username.to_string(), user_id: user_id.to_string(), group_id: group_id.to_string(), join_time: date_time, status: status,join_count: Some(0) }
    }

    pub fn get_status(&self) -> i8 {
        self.status
    }

    pub fn get_join_count(&self) -> i32 {
        self.join_count.unwrap()
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

//更改状态
pub async fn update_group_user_status(status : i8,group_id:&str,user_id:&str) -> Result<(),sqlx::Error>{
    let mysql = get_mysql();
    let sql = format!("
        update dd_group set
        status = {}
        where
        group_id = '{}' and user_id = '{}'
    ",status,group_id,user_id);
    sqlx::query(&sql).execute(mysql).await?;
    Ok(())
}

//更改验证数字
pub async fn update_group_user_join_count(join_count : i32,group_id:&str,user_id:&str) -> Result<(),sqlx::Error>{
    let mysql = get_mysql();
    let sql = format!("
        update dd_group set
        join_count = {}
        where
        group_id = '{}' and user_id = '{}'
    ",join_count,group_id,user_id);
    sqlx::query(&sql).execute(mysql).await?;
    Ok(())
}


//查询用户是否存在
pub async fn select_group_user(group_id:&str,user_id:&str) -> Result<Group,sqlx::Error> {
    let mysql = get_mysql();
    let sql = format!("
        select username,user_id,group_id,join_time,status,join_count from dd_group 
        where
        group_id = '{}' and user_id = '{}'
    ",group_id,user_id);
    let group:Group = sqlx::query_as(&sql).fetch_one(mysql).await?;
    Ok(group)
}