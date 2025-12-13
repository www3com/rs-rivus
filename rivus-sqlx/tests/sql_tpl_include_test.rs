use serde::Serialize;

#[derive(Serialize)]
struct EmptyParam {}

#[cfg(test)]
mod tests {

    use serde::Serialize;
    use rivus_sqlx::sql_tpl::engine::{remove_template, render_template};
    use crate::EmptyParam;

    #[test]
    fn test_include_template() {
        let cols_tpl_name = "cols";
        let cols_tpl_content = "id, name, age";
        let param = EmptyParam {};

        // Register "cols" template by rendering it once
        // (In a real app, maybe there's a separate API to register, but render_template does it)
        render_template(cols_tpl_name, cols_tpl_content, &param);

        let main_tpl_name = "selectUsers";
        let main_tpl_content = "select <include refid=\"cols\"/> from users";

        let (sql, _) = render_template(main_tpl_name, main_tpl_content, &param);

        assert_eq!(sql.trim(), "select id, name, age from users");

        remove_template(cols_tpl_name);
        remove_template(main_tpl_name);
    }

    #[test]
    fn test_include_nested() {
        let tpl1_name = "part1";
        let tpl1_content = "a";
        let param = EmptyParam {};
        render_template(tpl1_name, tpl1_content, &param);

        let tpl2_name = "part2";
        let tpl2_content = "<include refid=\"part1\"/>, b";
        render_template(tpl2_name, tpl2_content, &param);

        let main_name = "main";
        let main_content = "select <include refid=\"part2\"/>, c from t";
        let (sql, _) = render_template(main_name, main_content, &param);

        assert_eq!(sql.trim(), "select a, b, c from t");

        remove_template(tpl1_name);
        remove_template(tpl2_name);
        remove_template(main_name);
    }
}
