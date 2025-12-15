/**
 * 测试main方法执行
 * 这个类有标准的main方法
 */
public class MainTest {
    int a = 1;

    /**
     * 标准的main方法
     * 注意：暂时不能使用args参数（需要String对象支持）
     */
    public static void main(String[] args) {
        MainTest test = new MainTest(3);
        test.a = test.a + calculate();
        System.out.println(test.a);
    }

    public MainTest(int a) {
        this.a = a;
    }

    public MainTest() {
    }

    /**
     * 辅助方法：执行计算
     */
    public static int calculate() {
        int a = 10;
        int b = 20;
        int c = a + b;
        return c; // 返回30
    }
}
