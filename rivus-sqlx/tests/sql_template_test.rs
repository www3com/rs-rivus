use serde::Serialize;

#[derive(Serialize)]
struct Param<'a> {
    ids: Vec<i64>,
    name: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use rivus_sqlx::sql_tpl::engine::{remove_template, render_template};
    use rivus_sqlx::sql_tpl::value::SqlParam;
    use crate::Param;

    #[test]
    fn test_render_template_with_if_for() {
        let template_name = "getUsers";
        let tpl = r#"
select * from test
where 1=1
<if test="name != null"> and name = #{name}</if>
<for item="i" collection="ids" open=" and id in (" sep="," close=")">#{i}</for>
"#;

        let param = Param { ids: vec![1, 2, 3], name: Some("tom") };

        let (sql, params): (String, Vec<SqlParam>) = render_template(template_name, tpl, &param);

        let expected_sql = "
select * from test
where 1=1
 and name = ?
 and id in (?,?,?)";
        assert_eq!(sql.trim(), expected_sql.trim());

        assert_eq!(params.len(), 4);
        match &params[0] {
            SqlParam::String(s) => assert_eq!(s, "tom"),
            _ => panic!("param[0] should be String"),
        }
        match &params[1] {
            SqlParam::I64(v) => assert_eq!(*v, 1),
            _ => panic!("param[1] should be I64")
        }
        match &params[2] {
            SqlParam::I64(v) => assert_eq!(*v, 2),
            _ => panic!("param[2] should be I64")
        }
        match &params[3] {
            SqlParam::I64(v) => assert_eq!(*v, 3),
            _ => panic!("param[3] should be I64")
        }

        remove_template(template_name);
    }

    #[test]
    fn test_render_template_update() {
        let template_name = "updateTest";
        let tpl1 = "select 1 from test where id = #{id}";
        let param = SimpleParam { id: 1 };

        // First render
        let (sql1, _) = render_template(template_name, tpl1, &param);
        assert_eq!(sql1.trim(), "select 1 from test where id = ?");

        // Second render with different content
        let tpl2 = "select 2 from test where id = #{id}";
        let (sql2, _) = render_template(template_name, tpl2, &param);
        assert_eq!(sql2.trim(), "select 2 from test where id = ?");

        remove_template(template_name);
    }

    #[derive(Serialize)]
    struct SimpleParam {
        id: i64
    }

    #[test]
    fn test_render_template_simple() {
        let template_name = "getUsers2";
        let tpl = r#"select * from test where id = #{id}"#;

        let param = SimpleParam { id: 42 };
        let (sql, params) = render_template(template_name, tpl, &param);

        assert_eq!(sql.trim(), "select * from test where id = ?");
        assert_eq!(params.len(), 1);
        match &params[0] { SqlParam::I64(v) => assert_eq!(*v, 42), _ => panic!("param[0] should be I64") }

        remove_template(template_name);
    }
}
