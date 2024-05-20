using Sucrose.Mpv.NET.API.Interop;
using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    public partial class Mpv : IDisposable
    {
        public IMpvFunctions Functions
        {
            get => functions;
            set
            {
                Guard.AgainstNull(value);

                functions = value;
            }
        }

        public IMpvEventLoop EventLoop
        {
            get => eventLoop;
            set
            {
                Guard.AgainstNull(value);

                if (!value.IsRunning)
                {
                    value.Start();
                }

                eventLoop = value;
            }
        }

        public IntPtr Handle
        {
            get => handle;
            private set
            {
                if (value == IntPtr.Zero)
                {
                    throw new ArgumentException("Invalid handle pointer.", nameof(handle));
                }

                handle = value;
            }
        }

        private IMpvFunctions functions;
        private IMpvEventLoop eventLoop;
        private IntPtr handle;

        private bool disposed = false;

        public Mpv(string dllPath)
        {
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(dllPath, nameof(dllPath));

            Functions = new MpvFunctions(dllPath);

            InitialiseMpv();

            eventLoop = new MpvEventLoop(EventCallback, Handle, Functions);
            eventLoop.Start();
        }

        public Mpv(IMpvFunctions functions)
        {
            Functions = functions;

            InitialiseMpv();

            eventLoop = new MpvEventLoop(EventCallback, Handle, Functions);
        }

        public Mpv(IMpvFunctions functions, IMpvEventLoop eventLoop)
        {
            Functions = functions;

            EventLoop = eventLoop;

            InitialiseMpv();
        }

        internal Mpv(IntPtr handle, IMpvFunctions functions)
        {
            Handle = handle;

            Functions = functions;

            eventLoop = new MpvEventLoop(EventCallback, Handle, Functions);
            eventLoop.Start();
        }

        private void InitialiseMpv()
        {
            Handle = Functions.Create();
            if (Handle == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to create Mpv context.");
            }

            MpvError error = Functions.Initialise(Handle);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public long ClientAPIVersion()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.ClientAPIVersion();
        }

        public long ClientId()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.ClientId(Handle);
        }

        public string ErrorString(MpvError error)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.ErrorString(error);
        }

        public string ClientName()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.ClientName(Handle);
        }

        public Mpv CreateClient()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            IntPtr newHandle = Functions.CreateClient(Handle, out string name);
            if (newHandle == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to create new client.");
            }

            return new Mpv(newHandle, Functions);
        }

        public Mpv CreateWeakClient()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            IntPtr newHandle = Functions.CreateWeakClient(Handle, out string name);
            if (newHandle == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to create new weak client.");
            }

            return new Mpv(newHandle, Functions);
        }

        public void LoadConfigFile(string absolutePath)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            if (!Uri.TryCreate(absolutePath, UriKind.Absolute, out Uri _))
            {
                throw new ArgumentException("Path is not absolute.");
            }

            if (!File.Exists(absolutePath))
            {
                throw new FileNotFoundException("Config file not found.");
            }

            MpvError error = Functions.LoadConfigFile(Handle, absolutePath);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public long GetTimeUs()
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.GetTimeUs(Handle);
        }

        [Obsolete("Semi-deprecated in favour of SetProperty. Very few options still need to be set via SetOption.")]
        public void SetOption(string name, byte[] data, MpvFormat format = MpvFormat.ByteArray)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));
            Guard.AgainstNull(data, nameof(data));

            int dataLength = data.Length;
            IntPtr dataPtr = Marshal.AllocCoTaskMem(dataLength);

            try
            {
                Marshal.Copy(data, 0, dataPtr, dataLength);

                MpvError error = Functions.SetOption(Handle, name, format, dataPtr);
                if (error != MpvError.Success)
                {
                    throw MpvAPIException.FromError(error, Functions);
                }
            }
            finally
            {
                Marshal.FreeCoTaskMem(dataPtr);
            }
        }

        [Obsolete("Semi-deprecated in favour of SetPropertyString. Very few options still need to be set via SetOptionString.")]
        public void SetOptionString(string name, string data)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));
            Guard.AgainstNull(data, nameof(data));

            MpvError error = Functions.SetOptionString(Handle, name, data);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void Command(params string[] args)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNull(args, nameof(args));

            if (args.Length < 1)
            {
                throw new ArgumentException("Missing arguments.", nameof(args));
            }

            IntPtr argsPtr = MpvMarshal.GetComPtrForManagedUTF8StringArray(args, out IntPtr[] argsPtrs);
            if (argsPtr == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to convert string array to pointer array.");
            }

            try
            {
                MpvError error = Functions.Command(Handle, argsPtr);
                if (error != MpvError.Success)
                {
                    throw MpvAPIException.FromError(error, Functions);
                }
            }
            finally
            {
                MpvMarshal.FreeComPtrArray(argsPtrs);
                Marshal.FreeCoTaskMem(argsPtr);
            }
        }

        public void CommandString(string args)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(args, nameof(args));

            MpvError error = Functions.CommandString(Handle, args);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void CommandAsync(ulong replyUserData, params string[] args)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNull(args, nameof(args));

            if (args.Length < 1)
            {
                throw new ArgumentException("Missing arguments.", nameof(args));
            }

            IntPtr argsPtr = MpvMarshal.GetComPtrForManagedUTF8StringArray(args, out IntPtr[] argsPtrs);
            if (argsPtr == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to convert string array to pointer array.");
            }

            try
            {
                MpvError error = Functions.CommandAsync(Handle, replyUserData, argsPtr);
                if (error != MpvError.Success)
                {
                    throw MpvAPIException.FromError(error, Functions);
                }
            }
            finally
            {
                MpvMarshal.FreeComPtrArray(argsPtrs);
                Marshal.FreeCoTaskMem(argsPtr);
            }
        }

        public void AbortAsyncCommand(ulong replyUserData)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            MpvError error = Functions.AbortAsyncCommand(Handle, replyUserData);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void SetProperty(string name, byte[] data, MpvFormat format = MpvFormat.ByteArray)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));
            Guard.AgainstNull(data, nameof(data));

            if (data.Length < 1)
            {
                throw new ArgumentException("Data is empty.", nameof(data));
            }

            int dataLength = data.Length;
            IntPtr dataPtr = Marshal.AllocCoTaskMem(dataLength);

            try
            {
                Marshal.Copy(data, 0, dataPtr, dataLength);

                MpvError error = Functions.SetProperty(Handle, name, format, dataPtr);
                if (error != MpvError.Success)
                {
                    throw MpvAPIException.FromError(error, Functions);
                }
            }
            finally
            {
                Marshal.FreeCoTaskMem(dataPtr);
            }
        }

        public void SetPropertyString(string name, string value)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(value, nameof(value));

            MpvError error = Functions.SetPropertyString(Handle, name, value);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void SetPropertyAsync(string name, byte[] data, ulong replyUserData, MpvFormat format = MpvFormat.ByteArray)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));
            Guard.AgainstNull(data, nameof(data));

            if (data.Length < 1)
            {
                throw new ArgumentException("Data is empty.", nameof(data));
            }

            int dataLength = data.Length;
            IntPtr dataPtr = Marshal.AllocCoTaskMem(dataLength);

            try
            {
                Marshal.Copy(data, 0, dataPtr, dataLength);

                MpvError error = Functions.SetPropertyAsync(Handle, replyUserData, name, format, dataPtr);
                if (error != MpvError.Success)
                {
                    throw MpvAPIException.FromError(error, Functions);
                }
            }
            finally
            {
                Marshal.FreeCoTaskMem(dataPtr);
            }
        }

        public void SetPropertyLong(string name, long data)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            byte[] dataBytes = BitConverter.GetBytes(data);
            SetProperty(name, dataBytes, MpvFormat.Int64);
        }

        public void SetPropertyDouble(string name, double data)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            byte[] dataBytes = BitConverter.GetBytes(data);
            SetProperty(name, dataBytes, MpvFormat.Double);
        }

        public long GetPropertyLong(string name)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            MpvError error = Functions.GetPropertyLong(Handle, name, MpvFormat.Int64, out long value);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }

            return value;
        }

        public double GetPropertyDouble(string name)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            MpvError error = Functions.GetPropertyDouble(Handle, name, MpvFormat.Double, out double value);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }

            return value;
        }

        public string GetPropertyString(string name)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            IntPtr stringPtr = Functions.GetPropertyString(Handle, name);
            if (stringPtr == IntPtr.Zero)
            {
                throw new MpvAPIException("Failed to get property string, invalid pointer.");
            }

            try
            {
                return MpvMarshal.GetManagedUTF8StringFromPtr(stringPtr);
            }
            finally
            {
                Functions.Free(stringPtr);
            }
        }

        public void ObserveProperty(string name, MpvFormat format, ulong replyUserData)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            MpvError error = Functions.ObserveProperty(Handle, replyUserData, name, format);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, functions);
            }
        }

        public int UnobserveProperty(ulong registeredReplyUserData)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            int result = Functions.UnobserveProperty(Handle, registeredReplyUserData);
            if (result < 1)
            {
                MpvError error = (MpvError)result;
                throw MpvAPIException.FromError(error, functions);
            }

            return result;
        }

        public string EventName(MpvEventID eventID)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            return Functions.EventName(eventID);
        }

        public void RequestEvent(MpvEventID eventID, bool enabled)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            MpvError error = Functions.RequestEvent(Handle, eventID, enabled);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void RequestLogMessages(MpvLogLevel logLevel)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            string stringLogLevel = MpvLogLevelHelper.GetStringForLogLevel(logLevel);

            MpvError error = Functions.RequestLogMessages(Handle, stringLogLevel);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void HookAdd(string name, ulong replyUserData, int priority)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(name, nameof(name));

            MpvError error = Functions.HookAdd(Handle, replyUserData, name, priority);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void HookContinue(ulong id)
        {
            Guard.AgainstDisposed(disposed, nameof(Mpv));

            MpvError error = Functions.HookContinue(Handle, id);
            if (error != MpvError.Success)
            {
                throw MpvAPIException.FromError(error, Functions);
            }
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposed)
            {
                // NOTE
                // The order of disposal is important here. Functions.TerminateDestroy is
                // responsible for disposing of unmanaged resources on mpv's side.
                // Inside the Dispose method of the MpvFunctions object, Windows.FreeLibrary
                // is used to free the resources of the loaded mpv DLL.
                // Windows.FreeLibrary MUST COME AFTER Functions.TerminarDestroy

                // The event loop calls into mpv so we can't TerminateDestroy yet!
                if (disposing && EventLoop is IDisposable disposableEventLoop)
                {
                    disposableEventLoop.Dispose();
                }

                if (Handle != IntPtr.Zero)
                {
                    Functions?.TerminateDestroy(Handle);
                }

                if (disposing && Functions is IDisposable disposableFunctions)
                {
                    disposableFunctions.Dispose();
                }

                disposed = true;
            }
        }

        ~Mpv()
        {
            Dispose(false);
        }
    }
}