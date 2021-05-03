/**
 * @project DXLAB Which Way The Wind Blows
 * @author Small Multiples https://small/mu
 */

// Info overlay

function getWindowSize ( )
{
    return [ document.documentElement.clientWidth, document.documentElement.clientHeight ];
}

let info = document.getElementById("info_btn");
info.addEventListener ( "click", function ( e )
{
    let popup = document.getElementById("takeover");
    popup.classList.add("active");
} );

let infoClose = document.getElementById("popup_content");
infoClose.addEventListener ( "click", function ( e )
{
    let popup = document.getElementById("takeover");
    popup.classList.remove("active");
} );

// initialize video.js
var video = videojs('vis-player',  {
  fill: true,
  autoplay: true,
  fluid: true
});

video.ready(function() {
  video.controlBar.volumePanel.hide();
  video.controlBar.pictureInPictureToggle.hide();
  var promise = video.play();

  if (promise !== undefined) {
    promise.then(function() {
      // Autoplay started!
    }).catch(function(error) {
      // Autoplay was prevented.
    });
  }
});

//load the marker plugin
video.markers({
  markerStyle: {
     'width':'8px',
     'background-color': 'red'
  },
  markers: [
      {time: 0, text: "Day 1"},

      // Original speed
      // {time: 3.2857142857, text: "Day 2"},
      // {time: 6.5714285714, text: "Day 3"},
      // {time: 9.8571428571, text: "Day 4"},
      // {time: 13.1428571428, text: "Day 5"},
      // {time: 16.4285714285, text: "Day 6"},
      // {time: 19.7142857142, text: "Day 7"}

      // 2x speed to smooth out video
      {time: 1.642857143, text: "Day 2"},
      {time: 3.285714286, text: "Day 3"},
      {time: 4.928571429, text: "Day 4"},
      {time: 6.571428571, text: "Day 5"},
      {time: 8.214285714, text: "Day 6"},
      {time: 9.857142857, text: "Day 7"}
  ]    
});


[...document.getElementsByClassName('day')].forEach(function(button) {
  button.onclick = function(){
    video.pause();
    video.src(button.dataset.source);
    video.load();
    video.play();

    document.getElementById("currentWeek").textContent = "Seven days from " + button.querySelector('.date').textContent;

    [...document.getElementsByClassName('active')][0].classList.remove("active");
    button.classList.add("active");
    
  }
})
