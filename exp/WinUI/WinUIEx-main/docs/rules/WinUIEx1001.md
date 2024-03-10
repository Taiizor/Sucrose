## WinUIEX1001: The member will always be null.

The member will always be null and should not be used. This API was there due to UWP API surface area requirements, but is not needed for WinUI, and thus was never implemented.

|Item|Value|
|-|-|
|Category|Usage|
|Enabled|True|
|Severity|Warning|
|CodeFix|False|
---

One example is `Window.Current`: WinUI 3 does not have the notion of a current window. If you're interested in getting a reference to you main window, you could add a property to your `App` class that returns the main window, and set that property when you create the window.

### References

 - [Windows Runtime APIs not supported in desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-supported-api)
 - [Window.Current is null](https://github.com/microsoft/microsoft-ui-xaml/issues/4177)
 - [Deprecate and hide from Intellisense APIs that always return null, like Window.Current](https://github.com/microsoft/WindowsAppSDK/issues/1660)