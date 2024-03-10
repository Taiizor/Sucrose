using System;

namespace WinUIUnitTests
{
    public static class UITestHelper
    {
        public static Task RunUITest(Func<ContentControl, Task> action)
        {
            TaskCompletionSource tcs = new TaskCompletionSource();
            bool ok = App.Window.DispatcherQueue.TryEnqueue(async () =>
            {
                ContentControl c = new ContentControl();
                App.Window.Content = c;
                try
                {
                    await action(c);
                    tcs.TrySetResult();
                }
                catch (System.Exception ex)
                {
                    tcs.TrySetException(ex);
                }
                finally
                {
                    c.Content = null;
                    App.Window.Content = null;
                }
            });
            if (!ok)
                tcs.TrySetException(new InvalidOperationException("Could now run test on UI thread"));
            return tcs.Task;
        }

        public static Task RunWindowExTest(Func<WindowEx, Task> action)
        {
            TaskCompletionSource tcs = new TaskCompletionSource();
            bool ok = App.Window.DispatcherQueue.TryEnqueue(async () =>
            {
                WindowEx window = new WindowEx();
                window.Activate();
                try
                {
                    await action(window);
                    tcs.TrySetResult();
                }
                catch (System.Exception ex)
                {
                    tcs.TrySetException(ex);
                }
                finally
                {
                    window.Close();
                }
            });
            if (!ok)
                tcs.TrySetException(new InvalidOperationException("Could now run test on UI thread"));
            return tcs.Task;
        }

        public static Task RunWindowTest(Func<Window, Task> action)
        {
            TaskCompletionSource tcs = new TaskCompletionSource();
            bool ok = App.Window.DispatcherQueue.TryEnqueue(async () =>
            {
                Window window = new Window();
                window.Activate();
                try
                {
                    await action(window);
                    tcs.TrySetResult();
                }
                catch (System.Exception ex)
                {
                    tcs.TrySetException(ex);
                }
                finally
                {
                    window.Close();
                }
            });
            if (!ok)
                tcs.TrySetException(new InvalidOperationException("Could not run test on UI thread"));
            return tcs.Task;
        }

        public static Task RunWindowTest(Action<Window> action)
        {
            TaskCompletionSource tcs = new TaskCompletionSource();
            bool ok = App.Window.DispatcherQueue.TryEnqueue(() =>
            {
                Window window = new Window();
                window.Activate();
                try
                {
                    action(window);
                    tcs.TrySetResult();
                }
                catch (System.Exception ex)
                {
                    tcs.TrySetException(ex);
                }
                finally
                {
                    window.Close();
                }
            });
            if (!ok)
                tcs.TrySetException(new InvalidOperationException("Could now run test on UI thread"));
            return tcs.Task;
        }
    }
}
