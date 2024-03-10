## WinUIEX1002: Dispatcher must be replaced with DispatcherQueue.

The Dispatcher will always return null, and has been replaced by the DispatcherQueue. This API was there due to UWP API surface area requirements, but is not used in WinUI, and has been replaced by the DispatcherQueue.

|Item|Value|
|-|-|
|Category|Usage|
|Enabled|True|
|Severity|Warning|
|CodeFix|False|
---

### Example

To address this, change your use of `Dispatcher` from:
```cs
Dispatcher.RunAsync(Windows.UI.Core.CoreDispatcherPriority.Normal, () =>
{
   // your code
});
```
to:
```cs
DispatcherQueue.TryEnqueue(Microsoft.UI.Dispatching.DispatcherQueuePriority.Normal, () =>
{
   // your code
});
```

### References
 - [Windows Runtime APIs not supported in desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-supported-api)
 - [Remove DependencyObject.Dispatcher and Window.Dispatcher](https://github.com/microsoft/microsoft-ui-xaml/issues/6027) 
 - [Dispatcher properties are now null](https://github.com/microsoft/microsoft-ui-xaml/issues/4164)