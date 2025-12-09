/**
 * 计算器示例 - 测试各种运算
 */
public class Calculator {

    // 加法
    public static int add(int a, int b) {
        return a + b;
    }

    // 减法
    public static int subtract(int a, int b) {
        return a - b;
    }

    // 乘法
    public static int multiply(int a, int b) {
        return a * b;
    }

    // 除法
    public static int divide(int a, int b) {
        return a / b;
    }

    // 复杂计算: (a + b) * (c - d)
    public static int complex(int a, int b, int c, int d) {
        int sum = a + b;
        int diff = c - d;
        return sum * diff;
    }

    // 测试常量折叠（编译器优化）
    public static int constantFolding() {
        return 10 + 20 + 30;  // 编译器会优化成 iconst 60
    }

    // 测试无优化版本
    public static int noOptimization() {
        int a = 10;
        int b = 20;
        int c = 30;
        return a + b + c;
    }
}
