~~~rust
        // 6. 创建新栈帧并设置参数
                let mut new_frame = Frame::new(method.max_locals, method.max_stack);
                for (i, arg) in args.into_iter().enumerate() {
                    new_frame.set_local(i, arg)?;
                }

                // 7. 递归执行被调用的方法
                let result = self.execute_method_in_frame(
                    &method.code,
                    &mut new_frame,
                    &method_ref.class_name,
                )?;

                // 8. 将返回值压入调用者栈
                if let Some(return_val) = result {
                    frame.push(return_val);
                }

                // 9. 更新PC
                frame.pc += 3;
~~~
Frame {
    pc
    operator_stack
    local_vars
}
隐式栈, 依赖Rust的栈,正常返回后pc存在当前栈帧里,所以天然递增就可以

