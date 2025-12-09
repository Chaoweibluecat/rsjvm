/**
 * 简单的Java类示例
 * 用于测试Class文件解析和基本字节码执行
 */
public class Simple {

    // 静态字段
    public static int counter = 0;

    // 实例字段
    private int value;

    /**
     * 简单的加法方法
     */
    public static int add(int a, int b) {
        return a + b;
    }

    /**
     * 简单的减法方法
     */
    public static int subtract(int a, int b) {
        return a - b;
    }

    /**
     * 乘法
     */
    public static int multiply(int a, int b) {
        return a * b;
    }

    /**
     * 除法
     */
    public static int divide(int a, int b) {
        return a / b;
    }

    /**
     * 测试局部变量
     */
    public static int testLocalVars() {
        int x = 10;
        int y = 20;
        int z = x + y;
        return z;
    }

    /**
     * 测试循环（简单）
     */
    public static int sum(int n) {
        int result = 0;
        for (int i = 0; i <= n; i++) {
            result += i;
        }
        return result;
    }

    /**
     * 构造函数
     */
    public Simple(int value) {
        this.value = value;
    }

    /**
     * 获取值
     */
    public int getValue() {
        return value;
    }

    /**
     * 设置值
     */
    public void setValue(int value) {
        this.value = value;
    }
}
