using CefSharp;
using CefSharp.Wpf;
using System.Diagnostics;
using System.IO;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Rect = CefSharp.Structs.Rect;
using Size = System.Windows.Size;
using SSECSERC = Sucrose.Shared.Engine.CefSharp.Extension.RelayCommand;

namespace Sucrose.Shared.Engine.CefSharp.Control
{
    internal class CefBrowser : ChromiumWebBrowser
    {
        [DllImport("kernel32.dll", EntryPoint = "RtlCopyMemory", SetLastError = false)]
        private static extern void CopyMemory(IntPtr dest, IntPtr src, uint count);

        [DllImport("kernel32.dll", EntryPoint = "RtlMoveMemory", ExactSpelling = true)]
        private static extern void CopyMemory(IntPtr dest, IntPtr src, IntPtr count);

        private static readonly PixelFormat PixelFormat = PixelFormats.Pbgra32;
        private static readonly int BytesPerPixel = PixelFormat.BitsPerPixel / 8;

        private volatile bool isTakingScreenshot = false;
        private Size? screenshotSize;
        private int oldFrameRate;
        private int ignoreFrames = 0;
        private TaskCompletionSource<InteropBitmap> screenshotTaskCompletionSource;
        private CancellationTokenRegistration? cancellationTokenRegistration;

        public ICommand ScreenshotCommand { get; set; }

        public CefBrowser() : base()
        {
            ScreenshotCommand = new SSECSERC(TakeScreenshot);
        }

        public Task<InteropBitmap> TakeScreenshot(Size? screenshotSize = null, int? frameRate = 1, int? ignoreFrames = 0, CancellationToken? cancellationToken = null)
        {
            if (screenshotTaskCompletionSource != null && screenshotTaskCompletionSource.Task.Status == TaskStatus.Running)
            {
                throw new Exception("Screenshot already in progress, you must wait for the previous screenshot to complete");
            }

            if (IsBrowserInitialized == false)
            {
                throw new Exception("Browser has not yet finished initializing or is being disposed");
            }

            if (IsLoading)
            {
                throw new Exception("Unable to take screenshot while browser is loading");
            }

            IBrowserHost browserHost = this.GetBrowserHost();

            if (browserHost == null)
            {
                throw new Exception("IBrowserHost is null");
            }

            screenshotTaskCompletionSource = new TaskCompletionSource<InteropBitmap>(TaskCreationOptions.RunContinuationsAsynchronously);

            if (cancellationToken.HasValue)
            {
                CancellationToken token = cancellationToken.Value;
                cancellationTokenRegistration = token.Register(() =>
                {
                    screenshotTaskCompletionSource.TrySetCanceled();

                    cancellationTokenRegistration?.Dispose();

                }, useSynchronizationContext: false);
            }

            if (frameRate.HasValue)
            {
                oldFrameRate = browserHost.WindowlessFrameRate;
                browserHost.WindowlessFrameRate = frameRate.Value;
            }

            if (screenshotSize.HasValue)
            {
                this.screenshotSize = screenshotSize;
            }
            else
            {
                const string script = "[document.body.scrollWidth, document.body.scrollHeight]";

                JavascriptResponse javascriptResponse = this.EvaluateScriptAsync(script).Result;

                if (javascriptResponse.Success)
                {
                    List<object> widthAndHeight = (List<object>)javascriptResponse.Result;

                    this.screenshotSize = new((int)widthAndHeight[0], (int)widthAndHeight[1]);
                }
            }

            this.isTakingScreenshot = true;
            //this.screenshotSize = screenshotSize;
            this.ignoreFrames = ignoreFrames.GetValueOrDefault() < 0 ? 0 : ignoreFrames.GetValueOrDefault();

            //Resize the browser using the desired screenshot dimensions
            //The resulting bitmap will never be rendered to the screen
            browserHost.WasResized();

            return screenshotTaskCompletionSource.Task;
        }

        protected override Rect GetViewRect()
        {
            if (isTakingScreenshot)
            {
                return new Rect(0, 0, (int)Math.Ceiling(screenshotSize.Value.Width), (int)Math.Ceiling(screenshotSize.Value.Height));
            }

            return base.GetViewRect();
        }

        protected override void OnPaint(bool isPopup, Rect dirtyRect, IntPtr buffer, int width, int height)
        {
            if (isTakingScreenshot)
            {
                //We ignore the first n number of frames
                if (ignoreFrames > 0)
                {
                    ignoreFrames--;
                    return;
                }

                //Wait until we have a frame that matches the updated size we requested
                if (screenshotSize.HasValue && screenshotSize.Value.Width == width && screenshotSize.Value.Height == height)
                {
                    int stride = width * BytesPerPixel;
                    int numberOfBytes = stride * height;

                    //Create out own memory mapped view for the screenshot and copy the buffer into it.
                    //If we were going to create a lot of screenshots then it would be better to allocate a large buffer
                    //and reuse it.
                    MemoryMappedFile mappedFile = MemoryMappedFile.CreateNew(null, numberOfBytes, MemoryMappedFileAccess.ReadWrite);
                    MemoryMappedViewAccessor viewAccessor = mappedFile.CreateViewAccessor();

                    CopyMemory(viewAccessor.SafeMemoryMappedViewHandle.DangerousGetHandle(), buffer, (uint)numberOfBytes);

                    //Bitmaps need to be created on the UI thread
                    Dispatcher.BeginInvoke((Action)(() =>
                    {
                        IntPtr backBuffer = mappedFile.SafeMemoryMappedFileHandle.DangerousGetHandle();
                        //NOTE: Interopbitmap is not capable of supporting DPI scaling
                        InteropBitmap bitmap = (InteropBitmap)Imaging.CreateBitmapSourceFromMemorySection(backBuffer, width, height, PixelFormat, stride, 0);
                        screenshotTaskCompletionSource.TrySetResult(bitmap);

                        isTakingScreenshot = false;
                        IBrowserHost browserHost = GetBrowser().GetHost();
                        //Return the framerate to the previous value
                        browserHost.WindowlessFrameRate = oldFrameRate;
                        //Let the browser know the size changes so normal rendering can continue
                        browserHost.WasResized();

                        viewAccessor?.Dispose();
                        mappedFile?.Dispose();

                        cancellationTokenRegistration?.Dispose();
                    }));
                }
            }
            else
            {
                base.OnPaint(isPopup, dirtyRect, buffer, width, height);
            }
        }

        private void TakeScreenshot()
        {
            TaskScheduler uiThreadTaskScheduler = TaskScheduler.FromCurrentSynchronizationContext();

            const string script = "[document.body.scrollWidth, document.body.scrollHeight]";

            this.EvaluateScriptAsync(script).ContinueWith((scriptTask) =>
            {
                JavascriptResponse javascriptResponse = scriptTask.Result;

                if (javascriptResponse.Success)
                {
                    List<object> widthAndHeight = (List<object>)javascriptResponse.Result;

                    Size screenshotSize = new((int)widthAndHeight[0], (int)widthAndHeight[1]);

                    TakeScreenshot(screenshotSize, ignoreFrames: 0).ContinueWith((screenshotTask) =>
                    {
                        if (screenshotTask.Status == TaskStatus.RanToCompletion)
                        {
                            try
                            {
                                InteropBitmap bitmap = screenshotTask.Result;

                                string tempFile = Path.GetTempFileName().Replace(".tmp", ".jpeg");

                                using (FileStream stream = new(tempFile, FileMode.Create))
                                {
                                    JpegBitmapEncoder encoder = new();

                                    encoder.Frames.Add(BitmapFrame.Create(bitmap));
                                    encoder.Save(stream);
                                }

                                Process.Start(new ProcessStartInfo
                                {
                                    UseShellExecute = true,
                                    FileName = tempFile
                                });
                            }
                            catch (Exception ex)
                            {
                                throw new Exception(ex.Message);
                            }
                        }
                        else
                        {
                            throw new Exception("Unable to capture screenshot");
                        }
                    }, uiThreadTaskScheduler);

                }
                else
                {
                    throw new Exception("Unable to obtain size of screenshot");
                }
            }, uiThreadTaskScheduler);
        }
    }
}