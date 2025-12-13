
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemFn, ItemStruct, ReturnType};

// 1. 处理 Struct 上的 #[sql] - 目前主要是为了不报错，也可以用来做标记
pub fn sql_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // 尝试解析为函数
    let input_clone = input.clone();
    if let Ok(item_fn) = syn::parse::<ItemFn>(input_clone) {
        return handle_fn(args, item_fn);
    }

    // 尝试解析为 Struct
    let input_clone = input.clone();
    if let Ok(item_struct) = syn::parse::<ItemStruct>(input_clone) {
        // 对于 struct 上的 sql，我们直接原样返回，不做修改
        // 如果需要获取 namespace 参数，可以在这里解析 args
        return TokenStream::from(quote! { #item_struct });
    }

    // 如果都不是，报错
    syn::Error::new_spanned(
        proc_macro2::TokenStream::from(input),
        "#[sql] must be applied to a struct or function",
    )
        .to_compile_error()
        .into()
}

// 2. 核心逻辑：处理函数的 #[sql]
fn handle_fn(args: TokenStream, mut func: ItemFn) -> TokenStream {
    // 解析宏的参数，例如 #[sql("list_user")] 中的 "list_user"
    struct SqlArgs {
        id: String,
    }

    impl syn::parse::Parse for SqlArgs {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let vars = syn::punctuated::Punctuated::<syn::Lit, syn::Token![,]>::parse_terminated(input)?;
            let id = if let Some(syn::Lit::Str(lit)) = vars.first() {
                lit.value()
            } else {
                "Unknown".to_string()
            };
            Ok(SqlArgs { id })
        }
    }

    let args = parse_macro_input!(args as SqlArgs);
    let sql_id = args.id;

    let fn_name_str = func.sig.ident.to_string();

    // 收集参数信息的代码片段
    let mut print_stmts = Vec::new();

    // 遍历函数参数
    for arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // 获取参数名 (例如 person, sex)
            let pat = &pat_type.pat;
            let arg_name_str = pat.to_token_stream().to_string();

            // 获取参数类型 (例如 Person, i32)
            let ty = &pat_type.ty;

            // 生成打印语句：名称、值、类型
            // 注意：这里使用 stringify! 也就是把类型编译期转字符串，或者使用 type_name
            print_stmts.push(quote! {
                println!(
                    "  [Param] Name: {}, Value: {:?}, Type: {}",
                    #arg_name_str,
                    #pat, // 这里直接引用变量，前提是变量实现了 Debug
                    std::any::type_name::<#ty>()
                );
            });
        }
    }

    // 模拟返回值逻辑
    // 根据函数签名的返回类型，我们需要构造一个默认的返回值
    // 题目中是 Result<Vec<...>>，我们构造 Ok(vec![])
    let default_return = match &func.sig.output {
        ReturnType::Type(_, _) => quote! { Ok(vec![]) },
        ReturnType::Default => quote! { () },
    };

    // 获取原始函数体（里面包含了 exec!() 调用）
    let stmts = &func.block.stmts;

    // 策略：我们在新函数体开头定义一个局部宏 exec!，然后保留用户的函数体（或者直接忽略用户的函数体由我们完全接管）
    // 鉴于题目代码里写了 exec!()，最优雅的方式是让 exec! 展开为我们的打印逻辑。

    let new_body = quote! {
        {
            // 定义局部宏 exec!，它捕获了外部的变量（参数）
            // 这种写法使得 exec! 只能在当前函数内部有效
            macro_rules! exec {
                () => {
                    {
                        println!("--------------------------------------------------");
                        // 1. 打印 Struct 名称 + 方法名
                        // 使用 std::any::type_name::<Self>() 获取当前 impl 块的结构体名称
                        // 如果是普通函数，Self 可能会报错，这里假设是在 impl 块中使用，或者通过 trait 兼容
                        // 为了兼容 standalone 函数，我们可以尝试用 Option 包装或者直接用 strict 模式
                        // 这里演示标准 impl 块下的用法：
                        let struct_name = std::any::type_name::<Self>();
                        // 简单的字符串处理去掉详细路径
                        let short_struct_name = struct_name.split("::").last().unwrap_or(struct_name);

                        println!("Executing SQL: {}::{} (ID: {})", short_struct_name, #fn_name_str, #sql_id);

                        // 2. 打印参数
                        #(#print_stmts)*

                        // 3. 返回模拟值
                        #default_return
                    }
                };
            }

            // 执行原本的代码块，原本的代码块里写了 exec!()，现在会调用上面的宏
            #(#stmts)*
        }
    };

    // 替换函数体
    func.block = syn::parse2(new_body).expect("Failed to parse new body");

    TokenStream::from(quote! { #func })
}
