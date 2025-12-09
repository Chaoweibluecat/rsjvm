/**
 * 经典的HelloWorld示例
 * 这个类需要System.out.println支持，目前无法运行
 * 但可以用来学习class文件结构
 */
public class HelloWorld {

    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }

    public static void greet(String name) {
        System.out.println("Hello, " + name + "!");
    }
}
