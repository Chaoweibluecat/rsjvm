/**
 * 测试不同大小的整数常量如何存储
 */
public class ConstantTest {
    // 小常量：-1到5，使用 iconst_<i>
    public static int testIconst() {
        return 3;  // iconst_3
    }

    // 字节范围：-128到127，使用 bipush
    public static int testBipush() {
        return 100;  // bipush 100
    }

    // 短整数范围：-32768到32767，使用 sipush
    public static int testSipush() {
        return 10000;  // sipush 10000
    }

    // 大整数：需要常量池，使用 ldc
    public static int testLdc() {
        return 100000;  // ldc #<index>
    }

    // 字符串：总是在常量池
    public static String testString() {
        return "Hello";  // ldc #<index>
    }
}
