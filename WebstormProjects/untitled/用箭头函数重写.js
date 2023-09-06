function ask(question, yes, no) {
    if (confirm(question)) yes();
    else no();
}

ask(
    "Do you agree?",
    function() { alert("You agreed."); },
    function() { alert("You canceled the execution."); }
);
/*用箭头函数重写下面的函数表达式：*/

ask(
    "Do you agree?",
    ()=> { alert("You agreed."); },
    ()=> { alert("You canceled the execution."); })