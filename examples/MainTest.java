/**
 * 测试main方法执行
 * 这个类有标准的main方法
 */
public class MainTest {
    /**
     * 标准的main方法
     * 注意：暂时不能使用args参数（需要String对象支持）
     */
    public static void main(String[] args) {
        // 简单的算术运算（不依赖标准库）
        int result = calculate();
        // System.out.println(result);  // 需要标准库支持，暂不可用
    }

    /**
     * 辅助方法：执行计算
     */
    public static int calculate() {
        int a = 10;
        int b = 20;
        int c = a + b;
        return c;  // 返回30
    }
}
