### **欠債系統 ( Cash )**

欠債系統能管理群組內的債務，讓群組成員可以更方便地互相借貸和追蹤債務。

* /cash look：查看當前存在的債務
* /cash add [debtor] [creditor] [debt] [ps] \(可選\)：增加債務
  + debtor：債務人 (標註某人)
  + creditor：債權人 (標註某人)
  + debt：債務金額
  + ps：備註
  + Ex：/cash add @Akiya @Eric 1000 晚餐錢 (Akiya 欠 Eric 1000 元, 備註：晚餐錢)
* /cash del [index]：移除債務
  + index：/cash look 之索引
  + Ex：/cash del 2 (刪除第二個債務)
