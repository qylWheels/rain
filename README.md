# Rain语言

## 简介

- Rain语言是一个玩具语言。

- Rain语言的特性是缺乏特性。

- Rain语言使你感到无聊。

## 先上手看看

- 首先新建一个后缀名为`.rain`的文件，叫它`id.rain`好了。

- 在`id.rain`中写入如下内容：

    ```
    let id = fn (x) -> x in id
    ```

- 有两种方式可以运行：

    1. 如果你下载的是release版，直接`rain id.rain`。

    2. 如果你下载的是源码，则要先[配置rustup工具链](https://www.rust-lang.org/zh-CN/tools/install)，然后`cargo run -- id.rain`。

- 两种运行方式的回显可能会有所差异，但你应该都能看到类似于下面的内容（`'2`也有可能是`'1`、`'3`或其他任何可能的单引号 + 数字形式，这都是正确的）：

    ```
    Type: '2 -> '2
    Value: <function>
    ```

- 非常奇怪的回显……让我们看看这里面有什么魔法。
     
## 程序即表达式

- 在Rain语言中，没有语句（Statement）这种东西。

- 用Rain语言写成的每一个程序都是一个表达式（Expression）。

- Rain语言中的表达式有哪些？

    - 数字

        - 可正可负。
  
        - 例：`114514`、`-1919810`。

    - 布尔值

        - 只有`true`和`false`。

    - 变量

        - 仅由字母组成，大小写敏感，**不能**为关键字。

        - 关键字有：`if`、`then`、`else`、`true`、`false`、`let`、`in`。

        - 例：`x`、`Aminoac`。

    - `let`表达式

        - 形如`let x = <expr1> in <expr2>`。其中`<expr1>`被**绑定**到`x`上，且`<expr2>`中所有出现`x`的地方都会被**替换**为`<expr1>`。

        - 例：`let x = 1 in x + 1`，该表达式的值为`2`。

    - `if`表达式

        - 形如`if <guard> then <expr1> else <expr2>`。其中`<guard>`应为`Bool`类型表达式，且`<expr1>`和`<expr2>`应有相同类型。

        - 例：`if 3 <= 4 then 1 + 1 else 2 + 2`，该表达式的值为`2`。

    - 二元表达式

        - 形如`<expr1> <binop> <expr2>`。其中`<binop>`只能为`+`、`*`、`<=`中的一种。

        - 例：`4 <= 2`、`4 + 5`、`1 * 0`。

    - 函数

        - 好玩的东西，但现在你可能不会这么觉得。

        - 例：`fn (x, y, z) -> x + y + z`。其中括号内为参数，`->`右侧为用于计算函数结果的表达式。

    - 函数调用

        - 又一个好玩的东西。

        - 例：`let add = fn (x, y, z) -> x + y + z in add(1, 2, 3)`，结果为`6`。

## 好玩在哪

- 看完上面的介绍是不是觉得十分甚至九分无聊？好在你坚持看到了这里，接下来好玩的才刚刚开始。

- 不可变性（immutability）

    - 在Rain语言中，一切都是不可变的。？？？，好吧，听起来更无聊、更不可理喻了。

- 绑定（binding）与遮蔽（shadowing）

    - 在上文对`let`表达式的介绍中，我加粗了“**绑定**”这个词，意在强调我们是把表达式**绑定**到了变量上，而不是**定义**了一个变量。

    - 这有什么区别？

        - 在C语言中，如果我们**定义**了一个变量，后续就不能再**定义**一个重名的变量，否则会报“重定义（redefine）”错误。

        - 而在Rain语言中，我们在把一个表达式**绑定**到一个变量后，还可以将另一个表达式**绑定**到同名变量，例如：

            ```
            let x = 1 in
            let x = 2 in
            ...
            ```
        
        - 这样一来，`x = 1`这个绑定会被`x = 2`遮蔽，后面用到`x`的代码将只知道`x = 2`。

        - 我们甚至不用理会新旧表达式的类型是否相容！

            ```
            let x = 1 in
            let x = fn (a) -> a + 1 in
            ...
            ```

        - 也就是说，当你想要给某个变量绑定一个新的表达式时，新表达式是什么都无所谓。只需要注意：后面的代码将只知道该变量绑定到了新表达式，而对于旧的绑定一无所知。而且这样一来，“不可变性”就不是什么问题了，因为你可以声明名称重复的变量。

- 函数 = 闭包（closure）

    - 你可能在别的语言里用过闭包。在这些语言里，闭包的写法往往与“普通函数”不同，例如

        - python

            ``` python
            closure = lambda x : x + 1  # closure

            def func(): # "normal" function
                ...
            ```
        
        - rust

            ``` rust
            closure = |x| x + 1;    // closure

            fn func() {     // "normal" function
                ...
            }
            ```

    - 但在Rain里，没有“普通函数”与闭包的区别，只有“函数”这个统一的概念。

        ```
        let func = fn (x) -> x + 1 in ...
        ```
    
    - 你可能会说：“那我缺的环境捕获这块谁来给我补啊”。请看代码：

        ```
        let x = 1 in
        let add = fn (y) -> x + y in
        add(99)
        ```
    
    - 运行结果如下，不信可以试试：

        ```
        Type: Int
        Value: 100
        ```
    
    - `add`函数没有名为`x`的参数，而`x`的值被捕获到了，说明Rain里这个身兼“普通函数”和闭包为一体的“函数”，的确具有环境捕获的功能。黑子说话！

    - 但要注意以下这种情况会阻止环境捕获：

        ```
        let x = 1 in
        let add = fn (x) -> x + 1 in
        add(3)
        ```
    
    - 这种情况下，`add`的参数`x`与第1行的`x`重名，这会导致`x = 1`不会被捕获到函数`add`内。具体原理见[Capture-Avoiding Substitution](https://cs3110.github.io/textbook/chapters/interp/substitution.html#capture-avoiding-substitution)。

- Partial application（我故意保留英文名，因为我觉得它的中文名不够直观）

    - 这位更是重量级。

    - 考虑下面这个函数：

        ```
        let add = fn (x, y) -> x + y in ...
        ```

    - 在常见的编程语言（C、Python等）里，定义函数时有几个形参，调用函数时就要传几个实参进去。但Rain不一样，Rain允许你传入的实参个数小于形参个数。接着上文的`add`函数，我们只传一个参数进去：

        ```
        let addtwo = add(2) in
        addtwo(1)
        ```
    
    - 结果为`3`。这是什么鬼？为什么`addtwo`是一个函数？为什么将`1`传入`addtwo`里后结果是`3`？

    - 这是“[柯里化（Currying）](https://cs3110.github.io/textbook/chapters/hop/currying.html)”在搞鬼。简单来说，`add`函数的类型是`Int -> Int -> Int`。为了方便，你可以把它看作`(Int -> Int) -> Int`，这代表它接受两个`Int`类型的参数且返回值是`Int`。如果只传一个参数给`add`，它会生成一个类型为`Int -> Int`的东西（从左往右数第一个`Int`消失了），这代表一个函数，这个函数接受一个`Int`并返回一个`Int`。正如我们看到的，`addtwo`就是这样一个函数，所以我们可以传一个参数给它，并最后产出一个`Int`。

    - 但答案为什么是`3`？当我们调用`add(2)`时，`2`这个值被**绑定**到了（没错，又是**绑定**）`add`函数的形参`x`上，过程如下：

        ```
        fn (x, y) -> x + y
                |
                |
        fn (2, y) -> 2 + y
                |
                |
        fn (y) -> 2 + y
        ```
    
    - 上面这些代码只是为了展示绑定的结果，而并非是一段合法的Rain代码。最后一行的`fn (y) -> 2 + y`被绑定到`addtwo`这个变量上，也即实际结果为`let addtwo = fn (y) -> 2 + y in addtwo(1)`，这样一切都说得通了。

- Poly type（我不知道它的中文名叫什么）

    - 回到我们最初的那个例子：

        ```
        let id = fn (x) -> x in id
        ```

    - 解释器推导出`id`的类型是`'a -> 'a`，意思是这个函数有一个输入、一个输出，且输入和输出的类型一致，而不管具体是什么类型。因此我们可以把任何类型的参数传给`id`：

        ```
        id(0)
        id(true)
        id(fn (x, y) -> x)
        id(let one = 1 in one + 1)
        ```

    - 我不打算在这里讲得太深，感兴趣的可以看看[这个](https://www.zybuluo.com/darwin-yuan/note/424724)。

- 类型推导

    - 你有没有我们自始至终都没有写类型注解？这是因为Rain可以自动帮你推导出表达式的类型。

    - 这是整个项目里工作量最大、踩坑最多的地方。实现完类型推导引擎后，我才发现要实现语言原型，还是用ML系语言（如ocaml）方便一些。

    - 你会对类型推导及Hindley-Milner算法感兴趣的：[Type Inference](https://cs3110.github.io/textbook/chapters/interp/inference.html)。

## 还有没有更多好玩的？

- 去学一门函数式编程课程吧，比如[CS 3110](https://cs3110.github.io/textbook/cover.html)。

- 学完后记得入坑程序语言理论（Programming Language Theory, PLT）:)：

    - [CS 242](https://stanford-cs242.github.io/f19/)

    - [steshaw的PLT相关资料收集](https://steshaw.org/plt/)
