## MediaPlayerElement

The `MediaPlayerElement` control is a direct port of [UWP's `MediaPlayerElement`](https://docs.microsoft.com/en-us/uwp/api/Windows.UI.Xaml.Controls.MediaPlayerElement?view=winrt-22621) and provides the ability to video in your application.
For full documentation on the control, refer to the UWP documentation: https://docs.microsoft.com/en-us/uwp/api/Windows.UI.Xaml.Controls.MediaPlayerElement

```xml
<winex:MediaPlayerElement
  Source="https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
  AreTransportControlsEnabled="True"  />
```


### Configuring Media Transport Controls
```xml
<winex:MediaPlayerElement x:Name="player"
  Source="https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
  AreTransportControlsEnabled="True"
  <winex:MediaPlayerElement.TransportControls>
    <winex:MediaTransportControls IsFullWindowButtonVisible="False" />
  </winex:MediaPlayerElement.TransportControls>
</winex:MediaPlayerElement>
```
Also see [UWP documentation on MediaTransportControls])https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.mediatransportcontrols?view=winrt-22621#hiding-showing-enabling-and-disabling-buttons),
and [Create custom transport controls](https://docs.microsoft.com/en-us/windows/apps/design/controls/custom-transport-controls)

#### Transport Control Limitations
There are still some outstanding limitations to this port of the MediaPlayerElement's Transport Controls. The following buttons are not fully functional yet:
  - `ZoomButton`
  - `FastForwardButton`
  - `RewindButton`
  - `PlaybackRateButton`
  - `FullWindowButton`
  - Closed caption buttons.

It is recommended to turn these off with the `IsVisible` properties until their features are implemented.
If you have a need for these today, I'd highly encourage you to consider sponsoring or contributing to WinUIEx.


![image](https://user-images.githubusercontent.com/1378165/177426047-3467c800-3ea9-4eb6-b67c-51ff466ee786.png)

