using System;

namespace WinUIEx.Messaging
{
    /// <summary>
    /// Event arguments for the <see cref="WindowMessageMonitor.WindowMessageReceived"/> event.
    /// </summary>
    public sealed class WindowMessageEventArgs : EventArgs
    {
        internal WindowMessageEventArgs(IntPtr hwnd, uint messageId, nuint wParam, nint lParam)
        {
            Message = new Message(hwnd, messageId, wParam, lParam);
        }

        /// <summary>
        /// The result after processing the message. Use this to set the return result, after also setting <see cref="Handled"/> to <c>true</c>.
        /// </summary>
        /// <seealso cref="Handled"/>
        public nint Result { get; set; }

        /// <summary>
        /// Indicates whether this message was handled and the <see cref="Result"/> value should be returned.
        /// </summary>
        /// <remarks><c>True</c> is the message was handled and the <see cref="Result"/> should be returned, otherwise <c>false</c> and continue processing this message by other subsclasses.</remarks>
        /// <seealso cref="Result"/>
        public bool Handled { get; set; }

        /// <summary>
        /// The Windows WM Message
        /// </summary>
        public Message Message { get; }

        internal WindowsMessages MessageType => (WindowsMessages)Message.MessageId;
    }
}
