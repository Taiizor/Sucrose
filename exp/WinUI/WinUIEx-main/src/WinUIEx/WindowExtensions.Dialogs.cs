using Microsoft.UI.Xaml;
using Windows.Storage.Pickers;

namespace WinUIEx
{
    public static partial class WindowExtensions
    {
        /// <summary>
        /// Initializes a new instance of the MessageDialog class with the provided window as a parent to display a titled message
        /// dialog that can be used to ask your user simple questions.
        /// </summary>
        /// <param name="window">Parent window</param>
        /// <param name="content">The message displayed to the user.</param>
        /// <param name="title">The title you want displayed on the dialog.</param>
        /// <returns>Message dialog</returns>
        public static Windows.UI.Popups.MessageDialog CreateMessageDialog(this Window window, string content, string title = "")
        {
            var dialog = new Windows.UI.Popups.MessageDialog(content, title);
            WinRT.Interop.InitializeWithWindow.Initialize(dialog, window.GetWindowHandle());
            return dialog;
        }

        /// <summary>
        /// Creates a new instance of a FileOpenPicker with the provided window as a parent.
        /// </summary>
        /// <param name="window">Parent window</param>
        /// <returns>FileOpenPicker</returns>
        public static FileOpenPicker CreateOpenFilePicker(this Window window)
        {
            FileOpenPicker openPicker = new FileOpenPicker();
            WinRT.Interop.InitializeWithWindow.Initialize(openPicker, window.GetWindowHandle());
            return openPicker;
        }

        /// <summary>
        /// Creates a new instance of a FileSavePicker with the provided window as a parent.
        /// </summary>
        /// <param name="window">Parent window</param>
        /// <returns>SaveFilePicker</returns>
        public static FileSavePicker CreateSaveFilePicker(this Window window)
        {
            FileSavePicker savePicker = new FileSavePicker();
            WinRT.Interop.InitializeWithWindow.Initialize(savePicker, window.GetWindowHandle());
            return savePicker;
        }
    }
}
