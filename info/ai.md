### **AI生成 ( OpenAI )**

Ai生成系統方便群組成員可以更方便地使用AI生成圖片和與ChatGPT進行單次對話。

* /chat [message] [public] \(可選\) [model]\(可選\)：與ChatGPT進行單次對話 (預設model: ChatGPT-4o-mini)
  + message：與ChatGPT對話的訊息
  + public：是否讓其他人可以看到回覆
  + model：選擇的模型
  + Ex：/chat 你好 true chatgpt-4o-mini (以chatgpt-4o-mini模型跟ChatGPT說你好，且讓他人看到回覆)
* /image [prompt] [public] \(可選\) [model] \(可選\)：生成圖片 (預設model: Dell-E-3)
  + prompt：生成圖片的參數條件
  + public：是否讓其他人可以看到回覆
  + model：選擇的模型
  + Ex：/image 一個動漫風的少女 false (讓Ai生成含有prompt的圖片，且不讓他人看到回覆)
* /model_list：查看目前可用的模型
  + Ex：/model_list (查看目前可用的模型)
