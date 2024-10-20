using Sucrose.Mpv.NET.API.Interop;

namespace Sucrose.Mpv.NET.API
{
    public class MpvEventLoop : IMpvEventLoop, IDisposable
    {
        public bool IsRunning { get => isRunning; private set => isRunning = value; }

        public Action<MpvEvent> Callback { get; set; }

        public IntPtr MpvHandle
        {
            get;
            private set
            {
                if (value == IntPtr.Zero)
                {
                    throw new ArgumentException("Mpv handle is invalid.");
                }

                field = value;
            }
        }

        public IMpvFunctions Functions
        {
            get;
            set
            {
                Guard.AgainstNull(value);

                field = value;
            }
        }

        private Task eventLoopTask;

        private bool disposed = false;
        private volatile bool isRunning;

        public MpvEventLoop(Action<MpvEvent> callback, IntPtr mpvHandle, IMpvFunctions functions)
        {
            Callback = callback;
            MpvHandle = mpvHandle;
            Functions = functions;
        }

        public void Start()
        {
            Guard.AgainstDisposed(disposed, nameof(MpvEventLoop));

            DisposeEventLoopTask();

            IsRunning = true;

            eventLoopTask = new Task(EventLoopTaskHandler);
            eventLoopTask.Start();
        }

        public void Stop()
        {
            Guard.AgainstDisposed(disposed, nameof(MpvEventLoop));

            IsRunning = false;

            if (Task.CurrentId == eventLoopTask.Id)
            {
                return;
            }

            // Wake up WaitEvent in the event loop thread
            // so we can stop it.
            Functions.Wakeup(MpvHandle);

            eventLoopTask.Wait();
        }

        private void EventLoopTaskHandler()
        {
            while (IsRunning)
            {
                IntPtr eventPtr = Functions.WaitEvent(MpvHandle, Timeout.Infinite);
                if (eventPtr != IntPtr.Zero)
                {
                    MpvEvent @event = MpvMarshal.PtrToStructure<MpvEvent>(eventPtr);
                    if (@event.Id != MpvEventID.None)
                    {
                        Callback?.Invoke(@event);
                    }
                }
            }
        }

        private void DisposeEventLoopTask()
        {
            eventLoopTask?.Dispose();
        }

        public void Dispose()
        {
            Dispose(true);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (disposing)
            {
                if (!disposed)
                {
                    Stop();

                    DisposeEventLoopTask();
                }

                disposed = true;
            }
        }
    }
}