### **提醒器 ( Reminder )**

提醒器能讓群組成員設置提醒，在指定時間提醒群組成員。

* /reminder [weekdays] [time] [message]：設置提醒
  - weekdays：每週的哪幾天要通知，格式：d, d, ..
  - time：提醒時間，格式：HH: MM
  - message：通知訊息
  - Ex：/reminder 1, 5, 7 07:30 起床吃飯 (在每週一、五、七早上7:30提醒起床吃飯)
* /rm_reminder [index]：移除提醒
  - index：/look之索引
  - Ex：/rm_reminder 2 (移除第二條提醒)
* /look：查看所有以設置的提醒
