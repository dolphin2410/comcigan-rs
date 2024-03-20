const scriptName = "test_bot";
const cmdArr = ["명령어", "시간표","질문"];

function findDelta(cmd) {
    if (cmd == "내일") {
        return +1
    } else if (cmd == "어제") {
        return -1
    } else if (cmd == "모레") {
        return +2
    } else {
        return 0
    }
}

function response(room, msg, sender, isGroupChat, replier, imageDB, packageName) {  
    let split = msg.split(" ");
    let commands = split[0]

    if (commands == "!시간표") {
        let delta = "오늘"
        if (split.length > 1) {
            delta = split[1]
        }
        replier.reply(loadTimetable(findDelta(delta), 1))
    } else if (commands.startsWith("!시간표") && commands != "!시간표") {
        let delta = "오늘"
        if (split.length > 1) {
            delta = split[1]
        }
        let ban = commands.substring(4)
        replier.reply(loadTimetable(findDelta(delta), ban))
    }
}

function loadTimetable(delta, ban) {  
    var data = org.jsoup.Jsoup.connect("https://comciganrs.shuttleapp.rs?delta=" + delta + "&ban=" + ban)
        .header("Content-Type", "application/json")
        .ignoreContentType(true)
        .ignoreHttpErrors(true)
        .get()

    var json = JSON.parse(data.text())
    var total = json.subjects_list.length

    var outputText = json.time_data + "의 시간표 정보입니다\n\n"

    for (var i = 0; i < total; i++) {
        outputText += (i +  1) + "교시: " + json.subjects_list[i] + "(" + json.teachers_list[i] + ")\n"
    }
   
   return outputText
}