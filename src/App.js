import "./App.css";
import { invoke } from "@tauri-apps/api/tauri";

function App() {
  function changeSubmitValue() {
    const status = gifts_status();
    const submit = document.getElementById("submit");
    switch (status) {
      case GiftsStatus.runable:
        submit.value = "登録開始";
        return;
      case GiftsStatus.convertable:
        submit.value = "ギフト番号を抽出";
        return;
      case GiftsStatus.null:
        submit.value = "ギフト番号未検出";
        return;
      default:
        return;
    }
  }

  const GiftsStatus = {
    runable: 0,
    convertable: 1,
    null: 2,
  };

  function gifts_status() {
    const gifts = get_gifts();

    if (gifts === null) {
      return GiftsStatus.null;
    }

    console.log(document.getElementById("gifts").value);
    if (gifts.join("\n") === document.getElementById("gifts").value) {
      console.log("same");
      return GiftsStatus.runable;
    } else {
      return GiftsStatus.convertable;
    }
  }

  const get_gifts = () => {
    let gifts = document
      .getElementById("gifts")
      .value.match(/^(?<=\s*)[^\s]{16}(?=$|\s)/gm);
    if (gifts === null) {
      return null;
    }

    gifts.forEach((gift) => (gift = gift.replace(/\s/g, "")));
    console.log(gifts);
    return gifts;
  };

  const callRust = () => {
    get_gifts().forEach((gift) => {
      document.getElementById("submit").value = "登録中";
      invoke("register_nanaco_gift", {
        message: {
          number: document.getElementById("nanaco_number").value,
          password: document.getElementById("password").value,
          gift: gift,
        },
      })
        .then((message) => {
          console.log("Ok", message);
          let textarea = document.getElementById("gifts");
          textarea.value = textarea.value.replace(
            gift,
            `${gift} -> ${message.message}`
          );
          console.log(textarea.value.match(/^[^\s]{16}$/gm))
          if (!textarea.value.match(/^[^\s]{16}$/gm)) {
            document.getElementById("submit").value = "登録完了";
          }
        })
        .catch((message) => {
          console.error("Error", message);
        });
    });
  };

  function gifts_convert() {
    const gifts = get_gifts();
    if (gifts === null) {
      return;
    } else {
      document.getElementById("gifts").value = gifts.join("\n");
    }
  }

  function handleSubmit(e) {
    e.preventDefault();
    const status = gifts_status();
    switch (status) {
      case GiftsStatus.runable:
        callRust();
        return;
      case GiftsStatus.convertable:
        gifts_convert();
        changeSubmitValue();
        return;
      case GiftsStatus.null:
        return;
      default:
        return;
    }
  }
  return (
    <div className="App">
      <header className="App-header">
        <h1>nanacho</h1>
        <form name="register_nanaco_gift" onSubmit={handleSubmit}>
          <label>
            <input
              type="text"
              placeholder="nanaco番号(ハイフンなし16桁)"
              name="nanaco_number"
              id="nanaco_number"
              maxLength="16"
              pattern="^[0-9]{16}$"
              required
            />
          </label>
          <br></br>
          <label>
            <input
              type="password"
              placeholder="パスワード"
              name="password"
              id="password"
              required
            />
          </label>
          <br></br>
          <label>
            <textarea
              placeholder="ギフト番号 or メール本文"
              name="gifts"
              id="gifts"
              required
              onChange={changeSubmitValue}
            />
          </label>
          <br></br>
          <input type="submit" id="submit" name="submit" value="登録開始" />
        </form>
      </header>
    </div>
  );
}

export default App;
