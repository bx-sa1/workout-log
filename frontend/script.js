

function padNum(c, n) {
  return n.toString().padStart(c, '0').slice(0-c);
}

document.addEventListener('submit', function (event) {
  var element = event.target;
  if(element.id === "add-action-form") {
    let form = element;
    event.preventDefault();
    const resp = {
      date: `${new Date().toISOString()}`,
      exercise: form.elements.exercise.value,
      progression: form.elements.progression.value,
      sets: parseInt(form.elements.sets.value),
      reps: parseInt(form.elements.reps.value),
      weight: parseInt(form.elements.weight.value),
      difficulty: form.elements.difficulty.value,
      notes: form.elements.notes.value,
    };

    add_workout(resp);
    toggle_overlay(null, '');
  } else if(element.id === "server-address-form") {
    let form = element;
    event.preventDefault();
    sessionStorage.setItem("server-address", form.elements.address.value);
    dispatchEvent(new Event('load'));
    toggle_overlay(null, '');
  }
});

window.addEventListener('load', function() {
  if(sessionStorage.getItem("server-address") == null) {
    toggle_overlay(null, "server-address");
  } else {
    get_workouts();
  }
});

function toggle_overlay(event, type, close_on_click=false) {
  let overlay = document.querySelector(".overlay");
  if(overlay.style.display === "block") {
    overlay.style.display = "";
    overlay.onclick = null;
  } else if(overlay.style.display === "") {
    overlay.style.display = "block";
    fill_overlay(event, type);
    overlay.onclick = (e) => { if(e.target == overlay) toggle_overlay(e, type, close_on_click); }
  }
}

function fill_overlay(event, type) {
  let overlay_header = document.querySelector('.overlay-header');
  let overlay_content = document.querySelector('.overlay-content');

  switch (type) {
    case 'add-workout': 
      overlay_header.innerHTML = "Add Workout";
      overlay_content.innerHTML = `
          <form id="add-action-form">
            <label for="exercise">Exercise: </label>
            <input id="exercise" name="exercise"><br>
            <label for="progression">Progression: </label>
            <input id="progression" name="progression"><br>
            <label for="sets">Sets: </label>
            <input id="sets" name="sets"><br>
            <label for="reps">Reps: </label>
            <input id="reps" name="reps"><br>
            <label for="weight">Weight: </label>
            <input id="weight" name="weight"><br>
            <label for="difficulty">Difficulty: </label>
            <input id="difficulty" name="difficulty"><br>
            <label for="notes">Notes: </label>
            <input id="notes" name="notes"><br>
            <input type="submit" value="Submit">
          </form>`;
      break;
    case "workout-info":
      let row = event.currentTarget.data;
      overlay_header.innerHTML = `${new Date(row.date)}`;
      overlay_content.innerHTML = `
          <div>
            Exercise: ${row.exercise}<br>
            Progression: ${row.progression}<br>
            Sets: ${row.sets}<br>
            Reps: ${row.reps}<br>
            Weight: ${row.weight}<br>
            Difficulty: ${row.difficulty}<br>
            Notes: <input value="${row.notes}"></input><br>
          </div>
          <button id="delete-workout" type="button" onclick="delete_workout('${new Date(row.date).toISOString()}')">Delete</button>`;
      break;
    case "server-address":
      overlay_header.innerHTML = "Server Address";
      overlay_content.innerHTML = `
        <form id="server-address-form">
          <label for="address">Address: </label>
          <input id="address" name="address">
          <input type="submit" value="Submit">
        </form>`;
      break;
  }
}

function add_workout(resp) {
  fetch("http://" + sessionStorage.getItem("server-address") + "/workout", {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(resp)
  })
  .then(r => r.json())
  .catch(e => console.error(e));
}

function get_workouts() {
  fetch("http://" + sessionStorage.getItem("server-address") + "/workouts")
    .then(r => r.json())
    .then(j => {
      let workout_data = document.getElementById('workouts');
      let body = workout_data.createTBody();
      for(let workout of j.result) {
        let row = body.insertRow(-1);
        
        row.data = workout;
        row.onclick = (e) => { toggle_overlay(e, "workout-info"); }

        let date = row.insertCell(-1);
        let exercise = row.insertCell(-1);
        let sets = row.insertCell(-1);
        let reps = row.insertCell(-1);

        date.innerHTML = new Date(workout.date);
        exercise.innerHTML = workout.exercise;
        sets.innerHTML = workout.sets;
        reps.innerHTML = workout.reps;
      }
    })
    .catch(e => console.error('Fetch error:', e));
}

function delete_workout(date) {
  fetch("http://" + sessionStorage.getItem("server-address") + `/workout?date=${date}`, {
    method: 'DELETE'
  })
  .then(r => r.json())
  .catch(e => console.error(e));
}

