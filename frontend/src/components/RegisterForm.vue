<template>
  <form>
    <label for="registration-token-input">Registration token:</label>
    <input id="registration-token-input" placeholder="AE7D23...">
    <label for="name-input">Name:</label>
    <input id="name-input" placeholder="John Jones" minlength="4">
    <label for="email-input">E-Mail:</label>
    <input id="email-input" placeholder="foo@bar.com">
    <label for="password-input">Password:</label>
    <input id="password-input" type="password" minlength="8">
    <label for="password-repeat-input">Repeat password:</label>
    <input id="password-repeat-input" type="password" minlength="8">
    <button>Register</button>
    <p class="error-text"></p>
  </form>
</template>
